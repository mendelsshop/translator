mod converter;

use core::fmt;

use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use ratatui_textarea::TextArea;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = TranslatorState {
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
pub struct TranslatorState<'a> {
    kind: Translator,
    status: Status,
    pub input_buffer: TextArea<'a>,
}
#[derive(Debug, Clone, Default)]
pub enum Translator {
    Translating {
        current: converter::Text,
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
fn run(mut terminal: DefaultTerminal, app: TranslatorState) -> Result<()> {
    loop {
        terminal.draw(render(&app))?;
        if matches!(event::read()?, Event::Key(_)) {
            // break Ok(());
        }
    }
}

fn render(app: &TranslatorState<'_>) -> impl FnOnce(&mut ratatui::Frame<'_>) {
    |frame: &mut Frame| {
        let size = frame.area();
        frame.render_widget(draw_main(), frame.area());
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(5),
                Constraint::Percentage(85),
                Constraint::Percentage(5),
                Constraint::Percentage(5),
            ])
            .margin(1)
            .split(frame.area());
        frame.render_widget(
            &app.input_buffer,
            *layout.get(2).expect("could not get area to draw"),
        );
        let status = draw_status(&app.status);
        frame.render_widget(status, *layout.get(3).expect("could not get area to draw"));
    }
}
