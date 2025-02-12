mod ui;

use std::{
    io::{self},
    time::{Duration, Instant},
};

//use color_eyre::Result;
//use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
            MouseEvent, MouseEventKind,
        },
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
    prelude::*,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    //let mut terminal = ratatui::init();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    let app_result = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    ratatui::restore();
    app_result
}

#[derive(Debug, PartialEq, Eq)]
enum Selected {
    None,
    Search,
    Tabel,
}

impl Selected {
    fn next(&self) -> Self {
        match self {
            Self::None => Self::Search,
            Self::Search => Self::Tabel,
            Self::Tabel => Self::None,
        }
    }
}

#[derive(Debug)]
pub struct App {
    scroll: u16,
    exit: bool,
    pub search_term: String,
    last_tick: Instant,
    selected: Selected,
    size: Rect,
}

impl Default for App {
    fn default() -> Self {
        Self {
            search_term: String::from(""),
            exit: false,
            scroll: 0,
            last_tick: Instant::now(),
            selected: Selected::None,
            size: Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
        }
    }
}

impl App {
    const TICK_RATE: Duration = Duration::from_millis(1);
    const MAX_SCROLL: u16 = 10;

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

    fn draw(&mut self, frame: &mut Frame) {
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
                Event::Mouse(mouse) => self.handle_mouse_event(mouse),
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(_) => {
                if self.size.x <= mouse.column
                    && self.size.width >= mouse.column
                    && self.size.y <= mouse.row
                    && self.size.height >= mouse.row
                {
                    if self.size.y <= mouse.row && mouse.row <= 3 {
                        self.selected = Selected::Search;
                    } else if mouse.row > 3 && mouse.row <= self.size.height - 5 {
                        self.selected = Selected::Tabel
                    } else if mouse.row > self.size.height - 5 {
                        self.selected = Selected::None
                    }
                }
            }
            MouseEventKind::Moved => {}
            MouseEventKind::ScrollUp => self.scroll = (self.scroll + 1) % (Self::MAX_SCROLL + 1),
            MouseEventKind::ScrollDown => {
                if self.scroll as i16 - 1 < 0 {
                    self.scroll = Self::MAX_SCROLL;
                } else {
                    self.scroll -= 1;
                }
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') if self.selected == Selected::None => self.exit(),
            KeyCode::Char(x) if self.selected == Selected::Search => self.search_term.push(x),
            KeyCode::Backspace if self.selected == Selected::Search => {
                self.search_term.pop();
            }
            KeyCode::Tab => self.selected = self.selected.next(),
            KeyCode::Esc => self.selected = Selected::None,
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("Rusty Planner".bold());
        let instructions = Line::from(vec![
            " Currently Active ".into(),
            format!("{:?} ", self.selected).blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        self.size = block.inner(area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(5),
            ])
            .split(block.inner(area));

        let search_bar = Paragraph::new(Text::from(vec![Line::from(vec![
            "Search: ".into(),
            self.search_term.clone().white(),
        ])]))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if self.selected == Selected::Search {
                    Color::Green
                } else {
                    Color::White
                })),
        );

        let calender = Paragraph::new("NOTHING")
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(if self.selected == Selected::Tabel {
                        Color::Green
                    } else {
                        Color::White
                    })),
            )
            .scroll((self.scroll, 0))
            .wrap(Wrap { trim: true });

        block.render(area, buf);
        search_bar.render(layout[0], buf);
        calender.render(layout[1], buf);
    }
}
