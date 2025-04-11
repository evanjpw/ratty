mod terminal;
mod renderer;
mod config;
mod platform;

use anyhow::Result;

fn main() -> Result<()> {
    println!("Starting Ratty - A Rust Terminal Emulator");
    
    // Initialize the terminal
    let mut terminal = terminal::Terminal::new()?;
    
    // Run the main application loop
    let result = terminal.run();
    
    // Clean up and exit
    terminal.cleanup()?;
    
    result
}