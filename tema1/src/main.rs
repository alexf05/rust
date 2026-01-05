use std::{env, process};

mod commands; // This will contain the individual command implementations

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <command> [args...]", args[0]);
        process::exit(-1); // Invalid command or not enough arguments
    }

    let command_name = &args[1];
    let command_args = &args[2..];

    let exit_code = match commands::dispatch_command(command_name, command_args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error: {}", e);
            -1 // Generic error for now, specific command errors will override
        }
    };

    process::exit(exit_code);
}
