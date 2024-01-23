#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use resource_monitor::{
    multi_threaded_implementation::run_multi_threaded_implementation,
    single_threaded_implementation::run_single_threaded_implementation
};

fn main() -> Result<(), eframe::Error> {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    
    //run_multi_threaded_implementation()

    run_single_threaded_implementation()
}