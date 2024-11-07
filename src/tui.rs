use std::io;
use std::time::Duration;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, symbols};
use ratatui::text::Span;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Axis, Block, Chart, Dataset, Borders, Gauge, GraphType, Paragraph};

use crate::coord::EcefCoord;
use crate::csv_reader::TelemetryRecord;

#[derive(Debug, Default)]
pub struct App {
    initial_window: [f64; 2],
    window_x: [f64; 2],
    window_y: [f64; 2],
    exit: bool,
    time_chunk_duration: u64,
    current_chunk: usize,
    chunks: Vec<Vec<TelemetryRecord>>,
    initial_time: u64,
    // Display fields
    avg_vel: f64,
    current_time: u64,
    current_alt: f64,
    altitude_points: Vec<(f64, f64)>,
}

impl App {
    pub fn new(chunks: Vec<Vec<TelemetryRecord>>) -> Self {
        App {
            initial_window: [0.0, 10.0],
            window_x: [0.0, 10.0],
            window_y: [0.0, 10.0],
            exit: false,
            // 1_000_000_000 is 1 second in nanoseconds
            time_chunk_duration: 1_000_000_000,
            current_chunk: 0,
            chunks,
            initial_time: 0,
            avg_vel: 0.0,
            current_time: 0,
            current_alt: 0.0,
            altitude_points: Vec::new(),
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                self.draw(frame)
            })?;

            // Get next chunk of results to show
            if self.current_chunk < self.chunks.len() {
                let chunk = self.chunks.get(self.current_chunk).unwrap();
                let last = chunk.iter().last().take().unwrap();

                if self.initial_time == 0 {
                    self.initial_time = chunk.iter().next().take().unwrap().timestamp_ns;
                }

                self.current_time = (last.timestamp_ns - self.initial_time) / self.time_chunk_duration;

                let ecef = EcefCoord { x: last.pos_x, y: last.pos_y, z: last.pos_z };
                let alt: f64 = ecef.to_geo().alt.round(); // Round to whole number for nicer display
                self.current_alt = alt;

                self.altitude_points.push((self.current_chunk as f64, alt / 1000.0)); // Convert to Km for graph

                // todo actually average the velocity over the chunk
                self.avg_vel = last.vel_x;
                //self.velocity_points.push((self.current_time, alt));

                // update window bounds for graph
                if self.current_chunk < 240 {
                    self.window_y[1] = (self.current_alt / 1000.0).round() + 5f64;
                }
                self.window_x[1] = self.current_chunk as f64 + 10f64;

                self.current_chunk += 1;
            }

            // 250 is 4/sec 4 hz. 200 is 5/sec 5 hz
            if event::poll(Duration::from_millis(200))? {
                self.handle_events()?;
            }
        }
        Ok(())
    }

    // Called to render the terminal ui
    fn draw(&self, frame: &mut Frame) {
        let layout_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(5),
                Constraint::Percentage(15),
                Constraint::Percentage(10),
                Constraint::Percentage(68),
            ])
            .split(frame.area());

        let layout_top_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ])
            .split(layout_rows[1]);

        let text_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(10),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(layout_top_row[1]);

        let title = Line::from(" Blue Origin New Shepard flight telemetry from flight NS-13 ".bold());
        let instructions = Line::from(vec![
            " Start over ".into(),
            "<Space>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        // block around entire frame
        frame.render_widget(block, frame.area());
        // frame.render_widget(Block::bordered().border_set(border::THICK), layout_rows[0]);

        // Row 1
        // Render ascii rocket
        frame.render_widget(Paragraph::new(" .'.\n |o|\n.'o'.\n|.-.|\n'   '").centered(), layout_top_row[0]); //layout[1]);

        // Show text fields
        let freq_txt = Line::from(vec![
            "Sim is running at 5 flight seconds per second".into(),
        ]);
        frame.render_widget(freq_txt.centered().bold(), text_rows[1]);

        let time_txt = Line::from(vec![
            "    Time: +".into(),
            self.current_time.to_string().into(),
            " seconds".into(),
        ]);
        frame.render_widget(time_txt.centered().bold(), text_rows[2]);

        let alt_txt = Line::from(vec![
            " Altitude: ".into(),
            self.current_alt.to_string().into(),
            " meters".into(),
        ]);
        frame.render_widget(alt_txt.centered().bold(), text_rows[3]);

        // Row 2: Show progress bar
        let cur_percent = (self.current_chunk as f32 / (self.chunks.len() as f32) * 100.0).round() as u16;
        let percent_gauge = Gauge::default()
            .block(Block::default().title("Percent through mission").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(cur_percent);
        frame.render_widget(percent_gauge, layout_rows[2]);

        // Row 3: Altitude chart
        // Labels for the X and Y axes
        let x_labels = vec![
            Span::styled(
                format!("{}", self.window_x[0]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", self.window_x[1]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ];

        let y_labels = vec![
            Span::styled(
                format!("{}", self.window_y[0]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}", ((self.window_y[0] + self.window_y[1]) * 0.25).round())),
            Span::raw(format!("{}", ((self.window_y[0] + self.window_y[1]) / 2.0).round())),
            Span::raw(format!("{}", ((self.window_y[0] + self.window_y[1]) * 0.75).round())),
            Span::styled(
                format!("{}", self.window_y[1]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ];

        let datasets = vec![
            Dataset::default()
                .name("Altitude")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.altitude_points),
        ];

        let chart = Chart::new(datasets)
            .block(Block::bordered())
            .x_axis(
                Axis::default()
                    .title("Seconds")
                    .style(Style::default().fg(Color::Gray))
                    .labels(x_labels)
                    .bounds(self.window_x),
            )
            .y_axis(
                Axis::default()
                    .title("km")
                    .style(Style::default().fg(Color::Gray))
                    .labels(y_labels)
                    .bounds(self.window_y),
            );

        frame.render_widget(chart, layout_rows[3]);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Backspace => println!("You pressed Backspace"),
            KeyCode::Char(' ') => self.reset_sim(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
    fn reset_sim(&mut self) {
        //! Reset the sim default values
        self.window_x = self.initial_window;
        self.window_y = self.initial_window;
        self.current_chunk = 0;
    }
}

/***************************
           Tests
***************************/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_key_event() -> io::Result<()> {
        // Test that ratatui reacts to button presses as you'd expect
        let mut app = App::default();
        app.handle_key_event(KeyCode::Char(' ').into());
        // todo how to test this

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);

        Ok(())
    }
}