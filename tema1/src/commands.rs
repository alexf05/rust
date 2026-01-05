use anyhow::{anyhow, Result};
use std::fs;
use std::io::{self, Read};
use std::os::unix::fs::{PermissionsExt, symlink}; // Import symlink here
use std::path::{Path, PathBuf};
use filetime::{set_file_times, FileTime};


pub fn dispatch_command(command_name: &str, args: &[String]) -> Result<i32> {
    match command_name {
        "pwd" => handle_pwd(args),
        "echo" => handle_echo(args),
        "cat" => handle_cat(args),
        "mkdir" => handle_mkdir(args),
        "mv" => handle_mv(args),
        "ln" => handle_ln(args),
        "rmdir" => handle_rmdir(args),
        "rm" => handle_rm(args),
        "ls" => handle_ls(args),
        "cp" => handle_cp(args),
        "touch" => handle_touch(args),
        "chmod" => handle_chmod(args),
        _ => Err(anyhow!("Invalid command: {}", command_name)),
    }
}

fn handle_pwd(args: &[String]) -> Result<i32> {
    if !args.is_empty() {
        return Err(anyhow!("pwd: too many arguments"));
    }
    match std::env::current_dir() {
        Ok(path) => {
            println!("{}", path.display());
            Ok(0)
        }
        Err(e) => Err(anyhow!("pwd: failed to get current directory: {}", e)),
    }
}

fn handle_echo(args: &[String]) -> Result<i32> {
    let mut no_newline = false;
    let mut print_args_start_index = 0;

    if let Some(arg) = args.get(0) {
        if arg == "-n" {
            no_newline = true;
            print_args_start_index = 1;
        }
    }

    let to_print = args[print_args_start_index..].join(" ");

    if no_newline {
        print!("{}", to_print);
    } else {
        println!("{}", to_print);
    }

    Ok(0)
}

fn handle_cat(args: &[String]) -> Result<i32> {
    if args.is_empty() {
        return Err(anyhow!("cat: missing file operand"));
    }

    for file_path in args {
        match fs::File::open(file_path) {
            Ok(mut file) => {
                let mut content = String::new();
                if let Err(e) = file.read_to_string(&mut content) {
                    eprintln!("cat: {}: {}", file_path, e);
                    return Ok(-20);
                }
                print!("{}", content);
            }
            Err(e) => {
                eprintln!("cat: {}: {}", file_path, e);
                return Ok(-20);
            }
        }
    }
    Ok(0)
}

fn handle_mkdir(args: &[String]) -> Result<i32> {
    if args.is_empty() {
        return Err(anyhow!("mkdir: missing operand"));
    }

    for dir_path in args {
        if let Err(e) = fs::create_dir_all(dir_path) {
            eprintln!("mkdir: cannot create directory '{}': {}", dir_path, e);
            return Ok(-30);
        }
    }
    Ok(0)
}

fn handle_mv(args: &[String]) -> Result<i32> {
    if args.len() != 2 {
        return Err(anyhow!("mv: missing file operand or too many arguments"));
    }

    let source = Path::new(&args[0]);
    let destination = Path::new(&args[1]);

    if let Err(e) = fs::rename(source, destination) {
        eprintln!("mv: cannot move '{}' to '{}': {}", source.display(), destination.display(), e);
        return Ok(-40);
    }
    Ok(0)
}

fn handle_ln(args: &[String]) -> Result<i32> {
    let mut symbolic = false;
    let mut path_args = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--symbolic" => {
                symbolic = true;
            }
            _ => {
                path_args.push(&args[i]);
            }
        }
        i += 1;
    }

    if path_args.len() != 2 {
        return Err(anyhow!("ln: missing file operand or too many arguments"));
    }

    let source = Path::new(path_args[0]);
    let link_name = Path::new(path_args[1]);

    if symbolic { // Given the problem description, we only care about symbolic links.
        if let Err(e) = symlink(source, link_name) { // Call symlink directly
            eprintln!("ln: failed to create symbolic link '{}' to '{}': {}", link_name.display(), source.display(), e);
            return Ok(-50);
        }
    } else {
        eprintln!("ln: only symbolic links are supported. Use -s or --symbolic.");
        return Ok(-50);
    }

    Ok(0)
}

fn handle_rmdir(args: &[String]) -> Result<i32> {
    if args.is_empty() {
        return Err(anyhow!("rmdir: missing operand"));
    }

    for dir_path in args {
        if let Err(e) = fs::remove_dir(dir_path) {
            eprintln!("rmdir: failed to remove directory '{}': {}", dir_path, e);
            return Ok(-60);
        }
    }
    Ok(0)
}

fn handle_rm(args: &[String]) -> Result<i32> {
    let mut recursive = false;
    let mut dir_only = false;
    let mut files_to_remove = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-r" | "-R" | "--recursive" => {
                recursive = true;
            }
            "-d" | "--dir" => {
                dir_only = true;
            }
            _ => {
                files_to_remove.push(&args[i]);
            }
        }
        i += 1;
    }

    if files_to_remove.is_empty() {
        return Err(anyhow!("rm: missing operand"));
    }

    let mut encountered_error = false;
    for path_str in files_to_remove {
        let path = Path::new(path_str);

        if path.is_dir() {
            if recursive {
                if let Err(e) = fs::remove_dir_all(path) {
                    eprintln!("rm: cannot remove directory '{}': {}", path.display(), e);
                    encountered_error = true;
                }
            } else if dir_only {
                if let Err(e) = fs::remove_dir(path) {
                    eprintln!("rm: cannot remove empty directory '{}': {}", path.display(), e);
                    encountered_error = true;
                }
            } else {
                eprintln!("rm: cannot remove directory '{}': Is a directory. Use -r or -d to remove directories.", path.display());
                encountered_error = true;
            }
        } else if path.is_file() || path.is_symlink() {
            if let Err(e) = fs::remove_file(path) {
                eprintln!("rm: cannot remove '{}': {}", path.display(), e);
                encountered_error = true;
            }
        } else {
            eprintln!("rm: cannot remove '{}': No such file or directory", path.display());
            encountered_error = true;
        }
    }

    if encountered_error {
        Ok(-70)
    } else {
        Ok(0)
    }
}

fn handle_ls(args: &[String]) -> Result<i32> {
    let mut show_all = false;
    let mut recursive = false;
    let mut paths_to_list = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => {
                show_all = true;
            }
            "-R" | "--recursive" => {
                recursive = true;
            }
            _ => {
                paths_to_list.push(PathBuf::from(&args[i]));
            }
        }
        i += 1;
    }

    if paths_to_list.is_empty() {
        paths_to_list.push(PathBuf::from("."));
    }

    let mut encountered_error = false;
    for path_to_list in paths_to_list {
        if path_to_list.is_file() {
            println!("{}", path_to_list.display());
            continue;
        }

        if recursive {
            if let Err(_) = ls_recursive(&path_to_list, show_all, &path_to_list) {
                encountered_error = true;
            }
        } else {
            if let Err(e) = ls_single_directory(&path_to_list, show_all) {
                eprintln!("ls: cannot access '{}': {}", path_to_list.display(), e);
                encountered_error = true;
            }
        }
    }

    if encountered_error {
        Ok(-80)
    } else {
        Ok(0)
    }
}

fn ls_single_directory(path: &Path, show_all: bool) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        if show_all || !file_name_str.starts_with('.') {
            println!("{}", file_name_str);
        }
    }
    Ok(())
}

fn ls_recursive(path: &Path, show_all: bool, base_path: &Path) -> Result<()> {
    if path.is_file() {
        println!("{}", path.strip_prefix(base_path).unwrap_or(path).display());
        return Ok(());
    }

    println!("{}:", path.display());
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        if show_all || !file_name_str.starts_with('.') {
            let full_path = path.join(&file_name);
            if full_path.is_dir() {
                if file_name_str != "." && file_name_str != ".." {
                    ls_recursive(&full_path, show_all, base_path)?;
                }
            } else {
                println!("{}", full_path.strip_prefix(base_path).unwrap_or(&full_path).display());
            }
        }
    }
    Ok(())
}

fn handle_cp(args: &[String]) -> Result<i32> {
    let mut recursive = false;
    let mut operands = Vec::new(); // Will hold source(s) and destination

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-R" | "-r" | "--recursive" => {
                recursive = true;
            }
            _ => {
                operands.push(&args[i]);
            }
        }
        i += 1;
    }

    if operands.len() < 2 {
        return Err(anyhow!("cp: missing file operand"));
    }

    let source_path_str = operands[0];
    let destination_path_str = operands[1];

    let source = PathBuf::from(source_path_str);
    let mut destination = PathBuf::from(destination_path_str);

    // If destination is an existing directory, append source name to it
    if destination.is_dir() {
        if let Some(file_name) = source.file_name() {
            destination.push(file_name);
        }
    }

    if source.is_dir() {
        if !recursive {
            eprintln!("cp: -r not specified; omitting directory '{}'", source.display());
            return Ok(-90);
        }
        if let Err(e) = copy_dir_recursive(&source, &destination) {
            eprintln!("cp: cannot copy directory '{}' to '{}': {}", source.display(), destination.display(), e);
            return Ok(-90);
        }
    } else if source.is_file() {
        if let Err(e) = fs::copy(&source, &destination) {
            eprintln!("cp: cannot copy '{}' to '{}': {}", source.display(), destination.display(), e);
            return Ok(-90);
        }
    } else {
        eprintln!("cp: cannot stat '{}': No such file or directory", source.display());
        return Ok(-90);
    }

    Ok(0)
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = destination.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

fn handle_touch(args: &[String]) -> Result<i32> {
    let mut access_only = false;
    let mut no_create = false;
    let mut modify_only = false;
    let mut files_to_touch = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" => access_only = true,
            "-c" | "--no-create" => no_create = true,
            "-m" => modify_only = true,
            _ => files_to_touch.push(&args[i]),
        }
        i += 1;
    }

    if files_to_touch.is_empty() {
        return Err(anyhow!("touch: missing file operand"));
    }

    let now = FileTime::now();
    let mut encountered_error = false;

    for file_path_str in files_to_touch {
        let path = Path::new(file_path_str);

        let metadata_res = fs::metadata(path);

        if metadata_res.is_err() {
            // File does not exist
            if no_create {
                continue; // Do not create if -c is specified
            }
            // Create the file
            if let Err(e) = fs::File::create(path) {
                eprintln!("touch: cannot touch '{}': {}", path.display(), e);
                encountered_error = true;
                continue;
            }
            // If created, times are already current, no need to set explicitly unless specified
        } else {
            // File exists
            let metadata = metadata_res?;
            let atime = FileTime::from_last_access_time(&metadata);
            let mtime = FileTime::from_last_modification_time(&metadata);

            let new_atime = if modify_only { atime } else { now };
            let new_mtime = if access_only { mtime } else { now };

            if let Err(e) = set_file_times(path, new_atime, new_mtime) {
                eprintln!("touch: cannot touch '{}': {}", path.display(), e);
                encountered_error = true;
            }
        }
    }

    if encountered_error {
        Ok(-100)
    } else {
        Ok(0)
    }
}

fn handle_chmod(args: &[String]) -> Result<i32> {
    if args.len() != 2 {
        return Err(anyhow!("chmod: missing operand or too many arguments"));
    }

    let mode_str = &args[0];
    let path = Path::new(&args[1]);

    let current_permissions = fs::metadata(path)?.permissions();
    let mut current_mode = current_permissions.mode();

    if mode_str.chars().all(char::is_numeric) {
        // Numeric mode
        let numeric_mode = u32::from_str_radix(mode_str, 8)
            .map_err(|_| anyhow!("chmod: invalid mode: '{}'", mode_str))?;
        current_mode = numeric_mode;
    } else {
        // Symbolic mode parsing
        let mut chars = mode_str.chars().peekable();
        let mut target_who_mask = 0;
        let mut op = ' '; // Default operator
        let mut perm_bits = 0;

        // Parse 'who' part (u, g, o, a)
        let mut found_who = false;
        while let Some(&c) = chars.peek() {
            match c {
                'u' => { target_who_mask |= 0o700; chars.next(); found_who = true; },
                'g' => { target_who_mask |= 0o070; chars.next(); found_who = true; },
                'o' => { target_who_mask |= 0o007; chars.next(); found_who = true; },
                'a' => { target_who_mask |= 0o777; chars.next(); found_who = true; },
                _ => break,
            }
        }
        if !found_who { // If no 'who' specified, default to 'a' (all)
            target_who_mask = 0o777;
        }

        // Parse operator (+ or -)
        if let Some(&c) = chars.peek() {
            if c == '+' || c == '-' {
                op = c;
                chars.next();
            } else {
                return Err(anyhow!("chmod: invalid symbolic mode operator: '{}'", c));
            }
        } else {
            return Err(anyhow!("chmod: missing symbolic mode operator"));
        }

        // Parse permissions (r, w, x)
        let mut found_perms = false;
        while let Some(&c) = chars.peek() {
            match c {
                'r' => { perm_bits |= 0o4; chars.next(); found_perms = true; },
                'w' => { perm_bits |= 0o2; chars.next(); found_perms = true; },
                'x' => { perm_bits |= 0o1; chars.next(); found_perms = true; },
                _ => return Err(anyhow!("chmod: invalid permission: '{}'", c)),
            }
        }
        if !found_perms {
            return Err(anyhow!("chmod: missing symbolic permissions"));
        }

        // Apply permissions based on operator
        let mut effective_perm_change = 0;

        // Calculate permission bits for user, group, other based on `perm_bits`
        let user_perm = (perm_bits & 0o4) << 6 | (perm_bits & 0o2) << 6 | (perm_bits & 0o1) << 6;
        let group_perm = (perm_bits & 0o4) << 3 | (perm_bits & 0o2) << 3 | (perm_bits & 0o1) << 3;
        let other_perm = perm_bits & 0o4 | perm_bits & 0o2 | perm_bits & 0o1;

        // Combine based on who_mask
        effective_perm_change |= user_perm & target_who_mask;
        effective_perm_change |= group_perm & target_who_mask;
        effective_perm_change |= other_perm & target_who_mask;
        
        if op == '+' {
            current_mode |= effective_perm_change;
        } else { // op == '-'
            current_mode &= !effective_perm_change;
        }
    }

    let new_permissions = fs::Permissions::from_mode(current_mode);

    if let Err(e) = fs::set_permissions(path, new_permissions) {
        eprintln!("chmod: cannot change permissions of '{}': {}", path.display(), e);
        return Ok(-25);
    }

    Ok(0)
}