use std::{env};
use std::fs;
use std::process::exit;


fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    if args.len() < 2 {
        panic!("2 argumente");
    }

    match args[1].as_str(){
        "pwd" => run_pwd(),
        "echo" => run_echo(&args),
        "cat" => run_cat(&args),
        "mkdir" => run_mkdir(&args),
        "mv" => run_mv(&args),
        "ln" => run_ln(&args),
        "rmdir" => run_rmdir(&args),
        _=> println!("no command")
    }
}

fn run_pwd() {
    match env :: current_dir() {
        Ok(path) => println!("{}" ,path.display()),
        Err(e) => println!("{}", e)
    }
}

fn run_echo(args : &[String]) { 
    if args.len() > 2 && args[2] == "-n" {
        if args.len() > 3 {
            println!("{}", args[3..].join(" "));
        }
    } else {
        if args.len() > 2 {
            println!("{}", args[2..].join(" "));
        }
        println!()
    }
}

fn run_cat(args : &[String]) {
    if args.len() > 2 {
        for filename in &args[2..] {
            match fs::read_to_string(filename) {
                Ok(file) =>  print!("{}",file),
                Err(e) => {
                    eprintln!("Error at cat {}, {}", e, filename);
                    exit(236);
                }

            }
            
        }
    }
}

fn run_mkdir(args : &[String]) {
    if args.len() > 2 {
        for file in &args[2..] {
            if let Err(e) = fs::create_dir(file) {
                eprintln!("Eror mkdir {}, {}", e, file);
                exit(-30);
            }
        }
    }
}

fn run_mv(args : &[String]) {
    if args.len() > 3 {
        if let Err(e) = fs::rename(args[2].to_string(), args[3].to_string()) {
            eprintln!("Error mv {}",e);
            exit(-40);
        }
    }
}

fn run_ln(args : &[String]) {
    if args.len() > 4 && args[2] == "-s"{
        // if let Err(e) = fs::symlink(args[3], args[4]) {
        //     eprintln!("Error sym_ln {}", e);
        // }
        println!("sylink idk")
    } else if args.len() > 3 {
        if let Err(e) = fs::hard_link(args[2].as_str(), args[3].as_str()) {
            eprintln!("Error hard_ln {}", e);
        }
    }
}

fn run_rmdir(args : &[String]) {
    if args.len() > 2 {
        for file in &args[2..] {
            if let Err(e) = fs::remove_dir(file) {
                eprintln!("Error rm_dir {}, {}", e, file);
            }
        }
    }
}