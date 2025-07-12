#![no_std]
#![no_main]

use lib::*;
use alloc::vec::Vec;

extern crate lib;

fn main() -> isize {
    println!("YatSenOS Shell - Welcome!");
    println!("Shell PID: {}", sys_get_pid());
    println!("Type 'help' for available commands.");
    println!("");
    
    loop {
        print!("ysos> ");
        
        // Read user input
        let input = stdin().read_line();
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        // Handle special exit commands
        if input == "exit" || input == "quit" {
            println!("Goodbye!");
            sys_exit(0);
        }
        
        // Parse and execute command
        execute_command(input);
    }
}

fn execute_command(input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    
    let command = parts[0];
    
    match command {
        "help" => show_help(),
        "list" | "ls" => list_apps(),
        "ps" => list_processes(),
        "whoami" => show_current_pid(),
        "run" => {
            if parts.len() < 2 {
                println!("Usage: run <program_name>");
                println!("Example: run hello");
                println!("Use 'list' to see available programs.");
            } else {
                run_program(parts[1]);
            }
        }
        "clear" => clear_screen(),
        "exit" | "quit" => {
            println!("Goodbye!");
            sys_exit(0);
        }
        _ => {
            println!("Unknown command: {}", command);
            println!("Type 'help' for available commands.");
        }
    }
}

fn show_help() {
    println!("YatSenOS Shell - Help");
    println!("=====================");
    println!("Available commands:");
    println!("  help          - Show this help message");
    println!("  list / ls     - List all available user programs");
    println!("  ps            - List all running processes");
    println!("  whoami        - Show current shell process ID");
    println!("  run <program> - Run a user program and wait for completion");
    println!("  clear         - Clear the screen");
    println!("  exit / quit   - Exit the shell");
    println!("");
    println!("Examples:");
    println!("  list          - See available programs");
    println!("  run hello     - Run the hello program");
    println!("  ps            - See running processes");
    println!("");
    println!("Student ID: 23336011: Tan Dongze!"); // TODO: Replace with your actual student ID
    println!("=====================");
}

fn show_current_pid() {
    let pid = sys_get_pid();
    println!("Current shell PID: {}", pid);
}

fn list_apps() {
    println!("Available user programs:");
    sys_list_app();
}

fn list_processes() {
    println!("Running processes:");
    sys_stat();
}

fn run_program(program_name: &str) {
    println!("Starting program: {}", program_name);
    
    // Spawn the program
    let pid = sys_spawn(program_name);
    
    if pid == 0 {
        println!("Failed to start program: {}", program_name);
        println!("Make sure the program name is correct. Use 'list' to see available programs.");
        return;
    }
    
    println!("Program started with PID: {}", pid);
    
    // Wait for the program to finish
    println!("Waiting for program to finish...");
    let exit_code = sys_wait_pid(pid);
    
    if exit_code == usize::MAX as isize {
        println!("Warning: Program {} is still running (PID: {})", program_name, pid);
    } else {
        println!("Program {} finished with exit code: {}", program_name, exit_code);
    }
}

fn clear_screen() {
    // Clear screen by printing enough newlines
    for _ in 0..50 {
        println!("");
    }
    println!("YatSenOS Shell - Screen Cleared");
}

entry!(main);

