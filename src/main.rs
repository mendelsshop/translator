mod converter;

mod structure;
use core::fmt;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use ratatui_textarea::TextArea;

use crate::structure::Cursor;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = AppState {
        input_buffer: {
            let mut area = TextArea::default();
            area.set_cursor_line_style(
                area.cursor_line_style()
                    .remove_modifier(Modifier::UNDERLINED),
            );
            area.set_cursor_style(area.cursor_style().remove_modifier(Modifier::REVERSED));
            area.set_block(Block::default().borders(Borders::TOP));
            area
        },
        ..Default::default()
    };
    let result = run(terminal, app);
    ratatui::restore();
    result
}
#[derive(Debug, Clone, Default)]
pub struct AppState<'a> {
    kind: AppStateKind,

    status: Status,
    pub input_buffer: TextArea<'a>,
}
impl AppState<'_> {
    fn in_editing_mode(&self) -> bool {
        matches!(
            self.kind,
            AppStateKind::Translating {
                translation_state: TranslationState::Editing,
                ..
            }
        )
    }
    fn in_normal_mode(&self) -> bool {
        !self.in_editing_mode()
    }
}
#[derive(Debug, Clone, Default)]
pub enum TranslationState {
    Editing,
    #[default]
    Normal,
}
#[derive(Debug, Clone, Default)]
pub enum AppStateKind {
    Translating {
        postion: Cursor,
        current: structure::Text,
        translation_state: TranslationState,
    },
    #[default]
    New,
}
fn draw_main<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(" Translator ")
        .border_style(Style::default().fg(Color::White))
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::White))
}
#[derive(Debug, Clone)]
pub enum Status {
    Ok(Option<String>),
    Error(String),
    Warning(String),
    Loading,
}
impl Default for Status {
    fn default() -> Self {
        Status::Ok(None)
    }
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Ok(s) => match s {
                Some(s) => write!(f, "Ok: {s}"),
                None => write!(f, "Ok"),
            },
            Status::Error(s) => write!(f, "Err {s}"),
            Status::Warning(s) => write!(f, "Warn {s}"),
            Status::Loading => write!(f, "Loading..."),
        }
    }
}
fn draw_status<'a>(status: &Status) -> Paragraph<'a> {
    Paragraph::new(vec![Line::from(Span::raw(status.to_string()))])
        .style(Style::default().fg(Color::LightCyan))
        .block(
            Block::default()
                .borders(Borders::TOP)
                // .borders(Borders::BOTTOM)
                .style(Style::default().fg(Color::White)),
        )
}
fn run(mut terminal: DefaultTerminal, app: AppState) -> Result<()> {
    let mut app = app;
    loop {
        terminal.draw(render(&app))?;
        let event = event::read()?;
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) if app.in_normal_mode() => {
                break Ok(());
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                if let AppStateKind::Translating {
                    translation_state: translation_state @ TranslationState::Editing,
                    ..
                } = &mut app.kind
                {
                    *translation_state = TranslationState::Normal
                }
            }
            _ if app.in_editing_mode() => {
                app.input_buffer.input(event);
            }
            _ => {}
        }
    }
}

fn render(app: &AppState<'_>) -> impl FnOnce(&mut ratatui::Frame<'_>) {
    |frame: &mut Frame| {
        let _size = frame.area();
        frame.render_widget(draw_main(), frame.area());
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(1),
                Constraint::Percentage(87),
                Constraint::Percentage(6),
                Constraint::Percentage(6),
            ])
            .margin(1)
            .split(frame.area());
        let body = match &app.kind {
            AppStateKind::Translating {
                current,
                translation_state: _,
                postion: _,
            } => &current.to_string(),
            AppStateKind::New => "Please load or create a new translation",
        };
        frame.render_widget(
            Paragraph::new(body),
            *layout.get(1).expect("could not get area to draw"),
        );
        frame.render_widget(
            &app.input_buffer,
            *layout.get(2).expect("could not get area to draw"),
        );
        let status = draw_status(&app.status);
        frame.render_widget(status, *layout.get(3).expect("could not get area to draw"));
    }
}
