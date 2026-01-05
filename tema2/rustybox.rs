use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Dacă nu avem argumente, nu facem nimic (sau ieșim cu eroare, conform logicii generale)
    if args.len() < 2 {
        process::exit(0);
    }

    let command = &args[1];
    let params = &args[2..];

    let result = match command.as_str() {
        "pwd" => cmd_pwd(),
        "echo" => cmd_echo(params),
        "cat" => cmd_cat(params),
        "mkdir" => cmd_mkdir(params),
        "mv" => cmd_mv(params),
        "ln" => cmd_ln(params),
        "rmdir" => cmd_rmdir(params),
        "rm" => cmd_rm(params),
        "ls" => cmd_ls(params),
        "cp" => cmd_cp(params),
        "touch" => cmd_touch(params),
        "chmod" => cmd_chmod(params),
        _ => {
            println!("Invalid command");
            Err(-1)
        }
    };

    match result {
        Ok(_) => process::exit(0),
        Err(code) => process::exit(code),
    }
}

// --- Implementarea Comenzilor ---

fn cmd_pwd() -> Result<(), i32> {
    env::current_dir()
        .map(|path| println!("{}", path.display()))
        .map_err(|_| -1) // pwd nu are cod specificat în enunț pentru eroare, folosim default
}

fn cmd_echo(args: &[String]) -> Result<(), i32> {
    let mut no_newline = false;
    let mut start_idx = 0;

    if let Some(first) = args.first() {
        if first == "-n" {
            no_newline = true;
            start_idx = 1;
        }
    }

    let content = args[start_idx..].join(" ");
    print!("{}", content);
    if !no_newline {
        println!();
    }
    Ok(())
}

fn cmd_cat(args: &[String]) -> Result<(), i32> {
    if args.is_empty() { return Err(-20); }
    for filename in args {
        let mut file = File::open(filename).map_err(|_| -20)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|_| -20)?;
        print!("{}", contents);
    }
    Ok(())
}

fn cmd_mkdir(args: &[String]) -> Result<(), i32> {
    if args.is_empty() { return Err(-30); }
    for dir in args {
        fs::create_dir(dir).map_err(|_| -30)?;
    }
    Ok(())
}

fn cmd_mv(args: &[String]) -> Result<(), i32> {
    if args.len() < 2 { return Err(-40); }
    let source = &args[0];
    let dest = &args[1];
    fs::rename(source, dest).map_err(|_| -40)
}

fn cmd_ln(args: &[String]) -> Result<(), i32> {
    if args.len() < 2 { return Err(-50); }
    
    let symbolic = args.contains(&String::from("-s")) || args.contains(&String::from("--symbolic"));
    // Filtram flag-urile pentru a gasi sursa si destinatia
    let clean_args: Vec<&String> = args.iter().filter(|a| !a.starts_with("-")).collect();
    
    if clean_args.len() < 2 { return Err(-50); }
    let source = clean_args[0];
    let target = clean_args[1];

    if symbolic {
        symlink(source, target).map_err(|_| -50)
    } else {
        fs::hard_link(source, target).map_err(|_| -50)
    }
}

fn cmd_rmdir(args: &[String]) -> Result<(), i32> {
    if args.is_empty() { return Err(-60); }
    for dir in args {
        fs::remove_dir(dir).map_err(|_| -60)?;
    }
    Ok(())
}

fn cmd_rm(args: &[String]) -> Result<(), i32> {
    let recursive = args.iter().any(|s| s == "-r" || s == "-R" || s == "--recursive");
    let dir_only = args.iter().any(|s| s == "-d" || s == "--dir");
    let targets: Vec<&String> = args.iter().filter(|s| !s.starts_with("-")).collect();

    if targets.is_empty() { return Err(-70); }

    for target in targets {
        let path = Path::new(target);
        if path.is_dir() {
            if recursive {
                fs::remove_dir_all(path).map_err(|_| -70)?;
            } else if dir_only {
                fs::remove_dir(path).map_err(|_| -70)?;
            } else {
                // Nu putem sterge directoare fara flag-uri
                return Err(-70);
            }
        } else {
            fs::remove_file(path).map_err(|_| -70)?;
        }
    }
    Ok(())
}

fn cmd_ls(args: &[String]) -> Result<(), i32> {
    let show_details = args.iter().any(|s| s == "-l");
    let all = args.iter().any(|s| s == "-a" || s == "--all");
    let recursive = args.iter().any(|s| s == "-R" || s == "--recursive");
    
    // Luam directoarele specificate sau "." daca nu e niciunul
    let mut targets: Vec<&String> = args.iter().filter(|s| !s.starts_with("-")).collect();
    let default_dot = String::from(".");
    if targets.is_empty() { targets.push(&default_dot); }

    for target in targets {
        let path = Path::new(target);
        if path.is_file() {
            println!("{}", target);
        } else if path.is_dir() {
            if recursive {
                visit_dirs(path, all).map_err(|_| -80)?;
            } else {
                let entries = fs::read_dir(path).map_err(|_| -80)?;
                for entry in entries {
                    let entry = entry.map_err(|_| -80)?;
                    let name = entry.file_name().into_string().map_err(|_| -80)?;
                    if all || !name.starts_with('.') {
                        println!("{}", name);
                    }
                }
            }
        } else {
             // Daca calea nu exista
            return Err(-80); 
        }
    }
    Ok(())
}

// Functie ajutatoare pentru ls recursiv
fn visit_dirs(dir: &Path, all: bool) -> io::Result<()> {
    if dir.is_dir() {
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap();
            
            if !all && name.starts_with('.') { continue; }
            
            // Afisam calea
            println!("{}", path.display());
            
            if path.is_dir() {
                visit_dirs(&path, all)?;
            }
        }
    }
    Ok(())
}

fn cmd_cp(args: &[String]) -> Result<(), i32> {
    let recursive = args.iter().any(|s| s == "-r" || s == "-R" || s == "--recursive");
    let targets: Vec<&String> = args.iter().filter(|s| !s.starts_with("-")).collect();

    if targets.len() < 2 { return Err(-90); }
    let source = Path::new(targets[0]);
    let dest_raw = Path::new(targets[1]);
    
    // Daca nu mentionam numele destinatiei (e un folder), copiem cu numele sursei
    let dest = if dest_raw.is_dir() {
        dest_raw.join(source.file_name().ok_or(-90)?)
    } else {
        dest_raw.to_path_buf()
    };

    if source.is_dir() {
        if !recursive { return Err(-90); }
        copy_dir_recursive(source, &dest).map_err(|_| -90)?;
    } else {
        fs::copy(source, dest).map_err(|_| -90)?;
    }
    Ok(())
}

// Functie ajutatoare pentru cp recursiv
fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

fn cmd_touch(args: &[String]) -> Result<(), i32> {
    let no_create = args.iter().any(|s| s == "-c" || s == "--no-create");
    let targets: Vec<&String> = args.iter().filter(|s| !s.starts_with("-")).collect();

    if targets.is_empty() { return Err(-100); }

    for target in targets {
        let path = Path::new(target);
        if path.exists() {
            // Rust std nu permite actualizarea timestamp-ului (utimens) usor.
            // Pentru tema, deschidem fisierul in mod write fara truncate pentru a simula accesul,
            // sau il ignoram conform limitarilor "simple".
            // Nota: Un touch real necesita syscall-uri libc.
            continue; 
        } else if !no_create {
            File::create(path).map_err(|_| -100)?;
        }
    }
    Ok(())
}

fn cmd_chmod(args: &[String]) -> Result<(), i32> {
    // chmod accepta format: [optiuni] MODE FILE
    // Presupunem ca args[0] este modul si args[1] fisierul
    if args.len() < 2 { return Err(-25); }
    let mode_str = &args[0];
    let path = Path::new(&args[1]);

    // Verificam daca e numeric
    if let Ok(octal) = u32::from_str_radix(mode_str, 8) {
        let permissions = fs::Permissions::from_mode(octal);
        fs::set_permissions(path, permissions).map_err(|_| -25)?;
    } else {
        // Implementare simplificata simbolica: u+x, a-r etc.
        // Format asteptat: [ugoa][+-][rwx]
        let chars: Vec<char> = mode_str.chars().collect();
        if chars.len() < 3 { return Err(-25); }
        
        // 1. Cine?
        let who_mask = match chars[0] {
            'u' => 0o700, 'g' => 0o070, 'o' => 0o007, 'a' => 0o777,
            _ => return Err(-25),
        };
        // 2. Operatie
        let add = match chars[1] {
            '+' => true, '-' => false,
            _ => return Err(-25),
        };
        // 3. Ce?
        let what_val = match chars[2] {
            'r' => 4, 'w' => 2, 'x' => 1,
            _ => return Err(-25),
        };

        // Calculam bitii shiftati in functie de 'who'
        let mut bit_change = 0;
        if who_mask & 0o700 != 0 { bit_change |= what_val << 6; }
        if who_mask & 0o070 != 0 { bit_change |= what_val << 3; }
        if who_mask & 0o007 != 0 { bit_change |= what_val; }

        let metadata = fs::metadata(path).map_err(|_| -25)?;
        let mut current_mode = metadata.mode();

        if add {
            current_mode |= bit_change;
        } else {
            current_mode &= !bit_change;
        }

        fs::set_permissions(path, fs::Permissions::from_mode(current_mode)).map_err(|_| -25)?;
    }

    Ok(())
}