mod ui;

use std::{
    io::{self},
    time::{Duration, Instant},
};

//use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::*,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug)]
pub struct App {
    scroll: u16,
    exit: bool,
    search_term: String,
    last_tick: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            search_term: String::from(""),
            exit: false,
            scroll: 0,
            last_tick: Instant::now(),
        }
    }
}

impl App {
    const TICK_RATE: Duration = Duration::from_millis(250);

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            if self.last_tick.elapsed() >= Self::TICK_RATE {
                self.last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let timeout = Self::TICK_RATE.saturating_sub(self.last_tick.elapsed());
        while event::poll(timeout)? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Tab => todo!(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("Rusty Planner".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(block.inner(area));

        let search_bar = Paragraph::new(Text::from(vec![Line::from(vec![
            "Search: ".into(),
            self.search_term.clone().black(),
        ])]))
        .block(Block::bordered().border_type(BorderType::Rounded));

        let calender = Paragraph::new("NOTHING")
            .block(Block::bordered().border_type(BorderType::Rounded))
            .scroll((self.scroll, 0))
            .wrap(Wrap { trim: true });

        block.render(area, buf);
        search_bar.render(layout[0], buf);
        calender.render(layout[1], buf);
    }
}
