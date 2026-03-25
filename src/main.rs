#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(
    clippy::use_self,
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc
)]
use itertools::Itertools;
mod converter;

mod structure;

use core::fmt;
use std::{fs::read_to_string, time::Duration};

use ansi_to_tui::IntoText;
use color_eyre::Result;
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

use crate::{converter::parse, structure::Commentary};

fn main() -> Result<()> {
    simple_file_logger::init_logger("translator", simple_file_logger::LogLevel::Trace)?;
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
    const fn in_editing_mode(&self) -> bool {
        matches!(
            self.kind,
            AppStateKind::Translating {
                translation_state: TranslationState::Editing,
                ..
            }
        )
    }
    const fn in_normal_mode(&self) -> bool {
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
        Self::Ok(None)
    }
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok(s) => match s {
                Some(s) => write!(f, "Ok: {s}"),
                None => write!(f, "Ok"),
            },
            Self::Error(s) => write!(f, "Err {s}"),
            Self::Warning(s) => write!(f, "Warn {s}"),
            Self::Loading => write!(f, "Loading..."),
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
fn run(mut terminal: DefaultTerminal, app: AppState<'_>) -> Result<()> {
    let mut app = app;

    let theme = ratatui_explorer::Theme::default().add_default_title();
    let mut file_explorer = FileExplorerBuilder::build_with_theme(theme).unwrap();
    loop {
        terminal.draw(render(&mut app, &file_explorer))?;
        // polling slows down the input too much either don't poll or tune the polling rate
        if true || event::poll(Duration::from_millis(500)).unwrap() {
            let event = event::read()?;
            match (event, &mut app.kind) {
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    }),
                    _,
                ) => {
                    if app.in_normal_mode() {
                        break Ok(());
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('l'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        postion,
                        sub_postion: _,
                        current,
                    },
                ) => {
                    log::trace!("l");
                    if current
                        .text
                        .get(postion.0)
                        .is_some_and(|line| postion.1 < line.text.len().saturating_sub(1))
                    {
                        log::trace!("l(active)");
                        postion.1 += 1;
                    }
                }

                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('h'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        postion,
                        sub_postion: _,
                        current: _,
                    },
                ) => {
                    log::trace!("h");
                    if postion.1 > 0 {
                        log::trace!("h(active)");
                        postion.1 -= 1;
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('k'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        postion,
                        sub_postion: _,
                        current: _,
                    },
                ) => {
                    log::trace!("k");
                    if postion.0 > 0 {
                        log::trace!("k(active)");
                        postion.0 -= 1;
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('j'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        postion,
                        sub_postion: _,
                        current,
                    },
                ) => {
                    log::trace!("j");
                    // TODO: it depends on how the last line ends(CLRF...)
                    if postion.0 < current.text.len().saturating_sub(2) {
                        postion.0 += 1;

                        log::trace!("j(active)",);
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('t'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        postion,
                        sub_postion: _,
                        current,
                    },
                ) => {
                    log::info!("t");
                    let line = &mut current.text[postion.0];
                    // make sure its not inside a word boundry
                    let line_position = get_line_position(postion, line);
                    line.commentary
                        .entry(line_position + 1)
                        .or_insert(Commentary {
                            sentence_translation: None,
                            description_paragraph: None,
                        })
                        .sentence_translation
                        .get_or_insert_with(|| format!("translation{postion:?}"));
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('d'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        postion,
                        sub_postion: _,
                        current,
                    },
                ) => {
                    let line = &mut current.text[postion.0];
                    // make sure its not inside a word boundry
                    let line_position = get_line_position(postion, line);
                    line.commentary
                        .entry(line_position + 1)
                        .or_insert(Commentary {
                            sentence_translation: None,
                            description_paragraph: None,
                        })
                        .description_paragraph
                        .get_or_insert_with(|| vec![format!("description{postion:?}")]);
                }

                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('D'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        postion,
                        sub_postion: _,
                        current,
                    },
                ) => {
                    let line = &mut current.text[postion.0];
                    let line_position = get_line_position(postion, line);
                    // make sure its not inside a word boundry
                    line.commentary
                        .entry(line_position)
                        .or_insert(Commentary {
                            sentence_translation: None,
                            description_paragraph: None,
                        })
                        .description_paragraph
                        .get_or_insert_with(|| vec![format!("description{postion:?}")]);
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc, ..
                    }),
                    AppStateKind::Translating {
                        translation_state: translation_state @ TranslationState::Editing,
                        postion: _,
                        current: _,
                        ..
                    },
                ) => *translation_state = TranslationState::Normal,
                (
                    event,
                    AppStateKind::Translating {
                        translation_state: TranslationState::Editing,
                        ..
                    },
                ) => {
                    app.input_buffer.input(event);
                }
                (event, _) => {
                    file_explorer.handle(&event)?;
                    if let Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }) = event
                        && !file_explorer.current().is_dir
                    {
                        let file = read_to_string(file_explorer.current().path.clone()).unwrap();

                        let current = parse(&file);
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

fn get_line_position(postion: &(usize, usize), line: &structure::Line) -> usize {
    line.words
        .get_key_value(&postion.1)
        .map_or(postion.1, |(range, _)| range.end)
}

fn render(
    app: &mut AppState<'_>,
    file_explorer: &FileExplorer,
) -> impl FnOnce(&mut ratatui::Frame<'_>) {
    |frame: &mut Frame<'_>| {
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
                    .map(|(i, mut line)| {
                        // Current behaviour is to hide any commentary not current line maybe have
                        // toggle to control
                        if i == postion.0 {
                            if line.text.is_empty() {
                                line.text.push(' ');
                            }
                            let column = current.text.get(postion.0).map_or(0, |text| {
                                (text.text.len().saturating_sub(1)).min(postion.1)
                            });

                            let (text, mut plain_text, prev_i) =
                                line.commentary.iter().sorted_by_key(|x| x.0).fold(
                                    (String::new(), line.text, 0),
                                    |(text, mut plain_text, prev_i), (i, commentary)| {
                                        log::trace!("{commentary:?} {column} {prev_i} {i}");
                                        let processing_text = plain_text.split_off(*i - prev_i);
                                        if postion.1 < *i {
                                            // this is buggy
                                            let column = column - prev_i;
                                            plain_text.insert_str(column + 1, "\x1b[0m");
                                            plain_text.insert_str(column, "\x1b[47;5m");
                                        }

                                        (
                                            format!(
                                                "{}{}{}{}{}",
                                                text,
                                                if text.is_empty() { "" } else { "\n" },
                                                plain_text,
                                                commentary
                                                    .sentence_translation
                                                    .as_ref()
                                                    .map_or("", String::as_str),
                                                commentary
                                                    .description_paragraph
                                                    .as_ref()
                                                    .map_or(String::new(), |text| {
                                                        format!("\n{}\n", text.join("\n"))
                                                    })
                                            ),
                                            processing_text,
                                            *i,
                                        )
                                    },
                                );
                            if postion.1 >= prev_i {
                                let column = column - prev_i;
                                plain_text.insert_str(column + 1, "\x1b[0m");
                                plain_text.insert_str(column, "\x1b[47;5m");
                            }
                            let seperator = if text.is_empty() && !plain_text.is_empty() {
                                ""
                            } else {
                                "\n"
                            };
                            text + seperator + &plain_text
                        } else {
                            line.text
                        }
                    })
                    .join("\n");
                frame.render_widget(
                    Paragraph::new(text.to_text().unwrap()),
                    *layout.get(1).expect("could not get area to draw"),
                );
            }
            AppStateKind::New => frame.render_widget_ref(
                file_explorer.widget(),
                *layout.get(1).expect("could not get area to draw"),
            ),
        }
        frame.render_widget(
            &app.input_buffer,
            *layout.get(2).expect("could not get area to draw"),
        );
        let status = draw_status(&app.status);
        frame.render_widget(status, *layout.get(3).expect("could not get area to draw"));
    }
}
