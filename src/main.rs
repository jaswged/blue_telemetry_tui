mod coord;
mod csv_reader;
mod tui;

use ratatui::prelude::Widget;
use std::error::Error;

use crate::csv_reader::{read_csv_and_chunk, TelemetryRecord};
use crate::tui::App;

fn main() -> Result<(), Box<dyn Error>> {
    // io::Result<()> { // -> Result<(), Box<dyn std::error::Error>>
    let file_path = "data/truth_fast.csv";
    let time_chunk_duration: u64 = 1_000_000_000;
    let chunks: Vec<Vec<TelemetryRecord>> = read_csv_and_chunk(file_path, time_chunk_duration)?;

    // Boiler plate Ratatui
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = App::new(chunks).run(&mut terminal);
    ratatui::restore();

    Ok(app_result?)
}
