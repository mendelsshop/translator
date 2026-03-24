use itertools::Itertools;
mod converter;

mod structure;

use core::fmt;
use std::{fs::read_to_string, time::Duration};

use ansi_to_tui::IntoText;
use color_eyre::{Result, owo_colors::OwoColorize};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, FrameExt, Paragraph},
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder};
use ratatui_textarea::TextArea;

use crate::converter::parse;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = AppState {
        input_buffer: {
            let mut area = TextArea::default();
            // area.set_cursor_line_style(
            //     area.cursor_line_style()
            //         .remove_modifier(Modifier::UNDERLINED),
            // );
            // area.set_cursor_style(area.cursor_style().remove_modifier(Modifier::REVERSED));
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
        postion: (usize, usize),
        sub_postion: usize,
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

    let theme = ratatui_explorer::Theme::default().add_default_title();
    let mut file_explorer = FileExplorerBuilder::build_with_theme(theme).unwrap();
    loop {
        terminal.draw(render(&mut app, &file_explorer))?;
        let event = event::read()?;
        if event::poll(Duration::from_millis(500)).unwrap() {
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) if app.in_normal_mode() => {
                    break Ok(());
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('l'),
                    ..
                }) => {
                    if let AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        postion,
                        sub_postion: _,
                        current,
                    } = &mut app.kind
                    {
                        // current.text.replace_range(*postion..(*postion + 7), "");
                        // current.text.replace_range(*postion + 1..(*postion + 5), "");
                        // current.text.insert_str(*postion + 2, "\x1b[0m");
                        // current.text.insert_str(*postion + 1, "\x1b[47;5m");

                        if current
                            .text
                            .get(postion.0)
                            .is_some_and(|t| postion.1 <= t.len())
                        {
                            postion.1 += 1;
                        }
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('h'),
                    ..
                }) => {
                    if let AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        postion,
                        sub_postion: _,
                        current: _,
                    } = &mut app.kind
                    {
                        // current.text.replace_range(*postion..(*postion + 7), "");
                        // current.text.replace_range(*postion + 1..(*postion + 5), "");
                        // current.text.insert_str(*postion, "\x1b[0m");
                        // current.text.insert_str(*postion - 1, "\x1b[47;5m");
                        if postion.1 > 0 {
                            postion.1 -= 1;
                        }
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('d'),
                    ..
                }) => {
                    if let AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        postion,
                        sub_postion: _,
                        current,
                    } = &mut app.kind
                    {
                        current
                            .description
                            .insert(*postion, format!("description{:?}", postion));
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => {
                    if let AppStateKind::Translating {
                        translation_state: translation_state @ TranslationState::Editing,
                        postion: _,
                        current: _,
                        ..
                    } = &mut app.kind
                    {
                        *translation_state = TranslationState::Normal
                    }
                }
                _ if app.in_editing_mode() => {
                    app.input_buffer.input(event);
                }
                _ => {
                    file_explorer.handle(&event)?;
                    if let Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }) = event
                        && !file_explorer.current().is_dir
                    {
                        let file = read_to_string(file_explorer.current().path.clone()).unwrap();

                        let current = parse(&file);
                        // current.text.insert_str(1, "\x1b[0m");
                        // println!("{}", current.text.escape_default());
                        // current.text.insert_str(0, "\x1b[47;5m");
                        app.kind = AppStateKind::Translating {
                            sub_postion: 0,
                            postion: (0, 0),
                            current,
                            translation_state: TranslationState::Normal,
                        };
                    }
                }
            }
        }
    }
}

fn render(
    app: &mut AppState<'_>,
    file_explorer: &FileExplorer,
) -> impl FnOnce(&mut ratatui::Frame<'_>) {
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
        match &mut app.kind {
            AppStateKind::Translating {
                current,
                translation_state: _,
                sub_postion: _,
                postion,
            } => {
                let text = current
                    .text
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(i, mut text)| {
                        if i == postion.0 {
                            text.insert_str(postion.1 + 1, "\x1b[0m");
                            text.insert_str(postion.1, "\x1b[47;5m");
                        }
                        text
                    })
                    .join("\n");
                frame.render_widget(
                    Paragraph::new(text.to_text().unwrap()),
                    *layout.get(1).expect("could not get area to draw"),
                )
            }
            AppStateKind::New => frame.render_widget_ref(
                file_explorer.widget(),
                *layout.get(1).expect("could not get area to draw"),
            ),
        };
        frame.render_widget(
            &app.input_buffer,
            *layout.get(2).expect("could not get area to draw"),
        );
        let status = draw_status(&app.status);
        frame.render_widget(status, *layout.get(3).expect("could not get area to draw"));
    }
}
