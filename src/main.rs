mod frame;
mod sash;
mod renderer;
mod config;
mod platform;

use anyhow::Result;
use frame::{Frame, FrameInterface};

fn main() -> Result<()> {
    println!("Starting Ratty - A Rust Terminal Emulator");
    
    // Initialize the Frame (top-level application coordinator)
    let mut frame = Frame::initialize()
        .map_err(|e| anyhow::anyhow!("Failed to initialize application: {}", e))?;
    
    // Run the main application loop
    let result = frame.run()
        .map_err(|e| anyhow::anyhow!("Application runtime error: {}", e));
    
    // Clean up and exit gracefully
    if let Err(shutdown_error) = frame.shutdown() {
        eprintln!("Warning: Error during shutdown: {}", shutdown_error);
    }
    
    result
}