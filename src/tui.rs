use std::io;
use std::time::Duration;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, symbols};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::prelude::*;
// use ratatui::prelude::{Color, Line, Modifier, Style, Stylize, Text, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Axis, Block, Chart, Dataset, Borders, Gauge, GraphType, Paragraph};
// use ratatui::style::{Color, Modifier, Style, Stylize};

use crate::coord::EcefCoord;
use crate::csv_reader::TelemetryRecord;

#[derive(Debug, Default)]
pub struct App {
    window: [f64; 2],
    counter: u8,
    exit: bool,
    time_chunk_duration: u64,
    current_chunk: usize,
    chunks: Vec<Vec<TelemetryRecord>>,
    initial_time: u64,
    // Display fields
    avg_vel: f64,
    current_time: u64,
    current_alt: f64,
    altitude_points: Vec<(usize, f64)>,
    data1: Vec<(f64, f64)>,
}

// Counter ratatui example
impl App {
    pub fn new(chunks: Vec<Vec<TelemetryRecord>>) -> Self {
        App {
            window: [0.0, 20.0],
            counter: 0,
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
            data1: Vec::new(),
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
                self.avg_vel = last.vel_x;
                self.current_time = (last.timestamp_ns - self.initial_time) / self.time_chunk_duration;

                let ecef = EcefCoord { x: last.pos_x, y: last.pos_y, z: last.pos_z };
                let alt = ecef.to_geo().alt.round(); // Round to whole number for nicer display
                self.altitude_points.push((self.current_chunk, alt / 1000.0)); // Convert to Km for graph
                self.current_alt = alt;

                self.data1.push((self.current_chunk as f64, alt / 1000.0));

                // todo average the velocity over the chunk
                //self.velocity_points.push((self.current_time, alt));

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
        // frame.render_widget(self, frame.area());

        // from layout page
        // let layout = Layout::default()
        //     .direction(Direction::Vertical)
        //     .constraints(vec![
        //         Constraint::Percentage(50),
        //         Constraint::Percentage(50),
        //     ])
        //     .split(frame.area());

        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(60),
            ])
            .split(frame.area());

        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(75),
            ])
            .split(outer_layout[0]);

        frame.render_widget(Paragraph::new("Hello world!"), frame.area());

        let title = Line::from(" Blue Origin flight telemetry from flight NS-13 ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        // block around entire frame
        frame.render_widget(block, frame.area());

        // individual frame blocks
        // outer
        frame.render_widget(Paragraph::new("Top").centered(), outer_layout[0]);
        frame.render_widget(Block::bordered().border_set(border::THICK), outer_layout[0]);
        frame.render_widget(Block::bordered().border_set(border::THICK), outer_layout[1]);
        frame.render_widget(Block::bordered().border_set(border::THICK), outer_layout[2]);

        // inner
        frame.render_widget(Paragraph::new("Bottom").centered(), inner_layout[0]); //layout[1]);
        frame.render_widget(Block::bordered().border_set(border::THICK), inner_layout[0]);
        frame.render_widget(Block::bordered().border_set(border::THICK), inner_layout[1]);
        // frame.render_widget(Block::bordered().border_set(border::THICK), inner_layout[1]);
        
        // frame.render_widget(Paragraph::new("outer 0"), outer_layout[     0]);
        // frame.render_widget(Paragraph::new("inner 0").borders(Borders::ALL), inner_layout[0]);
        // frame.render_widget(Paragraph::new("inner 1").borders(Borders::ALL), inner_layout[1]);
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
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            KeyCode::Backspace => println!("You pressed Backspace"),
            KeyCode::Char(' ') => println!("You pressed Space"),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let title = Line::from(" Blue Origin flight telemetry from flight NS-13 ".bold());
        // let instructions = Line::from(vec![
        //     " Decrement ".into(),
        //     "<Left>".blue().bold(),
        //     " Increment ".into(),
        //     "<Right>".blue().bold(),
        //     " Quit ".into(),
        //     "<Q> ".blue().bold(),
        // ]);
        // let block = Block::bordered()
        //     .title(title.centered())
        //     .title_bottom(instructions.centered())
        //     .border_set(border::THICK);

        // let time_text = Text::from(vec![Line::from(vec![
        //     "New Shepard, Time +".into(),
        //     self.current_time.to_string().yellow(), // +x seconds
        //     " s".into(),
        // ])]);

        // let cur_percent = (self.current_chunk as f32 / (self.chunks.len() as f32) * 100.0).round() as u16;
        // let percent_gauge = Gauge::default()
        //     .block(Block::default().title("Percent through mission").borders(Borders::ALL))
        //     .gauge_style(Style::default().fg(Color::Cyan))
        //     .percent(cur_percent);

        // // Altitude chart
        // let chart_title = Line::from("Altitude Chart".bold());
        // let chart = Paragraph::new(vec![
        //     Line::from("Time vs Altitude: ".to_string()),
        //     Line::from(format!("|{}|", "-".repeat(self.avg_vel as usize / 10))),
        // ])
        //     .block(Block::default().title(chart_title).borders(Borders::ALL));

        // // Render the gauge
        // let gauge_area = Rect::new(area.x, area.y + 1, area.width, 3);
        // percent_gauge.render(gauge_area, buf);

        // // Render the chart below the gauge
        // let chart_area = Rect::new(area.x, area.y + 4, area.width, area.height - 4);
        // chart.render(chart_area, buf);

        // Paragraph::new(time_text)
        //     .centered()
        //     .block(block)
        //     .render(area, buf);

        // chart example
        // let x_labels = vec![
        //     Span::styled(
        //         format!("{}", self.window[0]),
        //         Style::default().add_modifier(Modifier::BOLD),
        //     ),
        //     Span::raw(format!("{}", (self.window[0] + self.window[1]) / 2.0)),
        //     Span::styled(
        //         format!("{}", self.window[1]),
        //         Style::default().add_modifier(Modifier::BOLD),
        //     ),
        // ];
        // let datasets = vec![
        //     Dataset::default()
        //         .name("data2")
        //         .marker(symbols::Marker::Dot)
        //         .style(Style::default().fg(Color::Cyan))
        //         .data(&self.data1),
        //     Dataset::default()
        //         .name("data3")
        //         .marker(symbols::Marker::Braille)
        //         .style(Style::default().fg(Color::Yellow))
        //         .data(&self.data1),
        // ];

        // let chart = Chart::new(datasets)
        //     .block(Block::bordered())
        //     .x_axis(
        //         Axis::default()
        //             .title("X Axis")
        //             .style(Style::default().fg(Color::Gray))
        //             .labels(x_labels)
        //             .bounds(self.window),
        //     )
        //     .y_axis(
        //         Axis::default()
        //             .title("Y Axis")
        //             .style(Style::default().fg(Color::Gray))
        //             .labels(["1".bold(), "0".into(), "110".bold()])
        //             .bounds([0.0, 1120.0]),
        //     ).render(area, buf);

        // frame.render_widget(chart, area);
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
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.counter, 1);

        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);

        Ok(())
    }
}