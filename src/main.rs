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

use crate::{
    converter::parse,
    structure::{CharLength, Commentary},
};

fn main() -> Result<()> {
    simple_file_logger::init_logger("translator", simple_file_logger::LogLevel::Trace)?;
    // currenlty we only set non bidi mode when we load a file, but for file picking since we rely
    // on ratatui_explorer which does do bidi we rely on the terminal emulator
    print!("\x1b[8l\x1b[1 k");
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

    print!("\x1b[8h\n\x1b[0 k\n");
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TranslationState {
    Editing,
    #[default]
    Normal,
}
#[derive(Debug, Clone)]
pub enum CommentaryPosition {
    Description(usize, usize),
    Translation(usize),
}
#[derive(Debug, Clone, Default)]
pub enum AppStateKind {
    Translating {
        position: (usize, usize),

        sub_position: Option<CommentaryPosition>,
        command_buffer: String,
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
                        position,
                        current,
                        sub_position,
                        ..
                    },
                ) => {
                    log::trace!("l");
                    if let Some(sub_position) = sub_position {
                        current.text[position.0]
                            .get_commentary(position.1)
                            .inspect(|commentary| match sub_position {
                                CommentaryPosition::Description(line, column)
                                    if commentary.description_paragraph.as_ref().is_some_and(
                                        |desc| {
                                            desc.get(*line).is_some_and(|desc_line| {
                                                *column
                                                    < desc_line
                                                        .as_str()
                                                        .char_len()
                                                        .saturating_sub(1)
                                            })
                                        },
                                    ) =>
                                {
                                    // TODO: also keep track of column number for string just the
                                    // shown character index using str::floor_char_boundary(index),
                                    // and str::ceil_char_boundary
                                    *column += 1;
                                }
                                CommentaryPosition::Translation(column)
                                    if commentary.sentence_translation.as_ref().is_some_and(
                                        |translation| {
                                            *column
                                                < translation.as_str().char_len().saturating_sub(1)
                                        },
                                    ) =>
                                {
                                    *column += 1;
                                }
                                _ => {}
                            });
                    } else if current
                        .text
                        .get(position.0)
                        .is_some_and(|line| position.1 < line.len.saturating_sub(1))
                    {
                        log::trace!("l(active)");
                        position.1 += 1;
                    }
                }

                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('h'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        position,
                        sub_position,
                        current,
                        ..
                    },
                ) => {
                    log::trace!("h");
                    match sub_position {
                        Some(CommentaryPosition::Description(line_pos, column)) => {
                            if *column > 0 {
                                let line = &current.text[position.0];
                                *column = (*column - 1)
                                    // if cursor was from previous line which was longer we have to go
                                    // back 2 b/c len is 1 based, and we where already at last column
                                    .min(
                                        line.get_commentary_unchecked(position.1)
                                            .description_paragraph
                                            .as_ref()
                                            .unwrap()[*line_pos]
                                            .len()
                                            .saturating_sub(2),
                                    );
                            }
                        }
                        Some(CommentaryPosition::Translation(column)) => {
                            if *column > 0 {
                                *column -= 1;
                            }
                        }
                        _ if position.1 > 0 => {
                            log::trace!("h(active)");
                            position.1 = (position.1 - 1)
                                // if cursor was from previous line which was longer we have to go
                                // back 2 b/c len is 1 based, and we where already at last column
                                .min(current.text[position.0].len.saturating_sub(2));
                        }
                        _ => (),
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('k'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        position,
                        sub_position,
                        ..
                    },
                ) => {
                    log::trace!("k");
                    if let Some(CommentaryPosition::Description(line, _)) = sub_position {
                        if *line > 0 {
                            *line -= 1;
                        }
                    } else if position.0 > 0 {
                        // if editing translation and press k then you exit translation (b/c not
                        // more than one line)
                        *sub_position = None;
                        log::trace!("k(active)");
                        position.0 -= 1;
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('j'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        position,
                        current,
                        sub_position,
                        ..
                    },
                ) => {
                    log::trace!("j");
                    if let Some(CommentaryPosition::Description(line_pos, _)) = sub_position {
                        // TODO: maybe don't index and actually check that those indices exist
                        let line = &current.text[position.0];
                        if line
                            .get_commentary_unchecked(position.1)
                            .description_paragraph
                            .as_ref()
                            .is_some_and(|desc| *line_pos < desc.len().saturating_sub(1))
                        {
                            *line_pos += 1;
                        }
                    }
                    // TODO: it depends on how the last line ends(CLRF...)
                    else if position.0 < current.text.len().saturating_sub(2) {
                        position.0 += 1;
                        // if editing translation and press j then you exit translation (b/c not
                        // more than one line)
                        // if editing translation and press j then you exit translation
                        *sub_position = None;

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
                        position,
                        current,
                        command_buffer,
                        sub_position,
                        ..
                    },
                ) => {
                    log::info!("t");
                    if command_buffer == " " {
                        command_buffer.clear();
                        let line = &mut current.text[position.0];
                        // make sure its not inside a word boundary
                        let line_position = get_line_position(position, line, true);
                        line.commentary
                            .entry(line_position)
                            .or_insert(Commentary {
                                sentence_translation: None,
                                description_paragraph: None,
                            })
                            .sentence_translation
                            .get_or_insert_default();
                        if position.1 < line.len {
                            position.1 = line_position;
                        }
                        *sub_position = Some(CommentaryPosition::Translation(0));
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('d'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        position,
                        current,
                        sub_position,
                        command_buffer,
                        ..
                    },
                ) => {
                    log::info!("d");
                    if command_buffer == " " {
                        command_buffer.clear();
                        let line = &mut current.text[position.0];
                        // make sure its not inside a word boundary
                        let line_position = get_line_position(position, line, true);
                        line.commentary
                            .entry(line_position)
                            .or_insert(Commentary {
                                sentence_translation: None,
                                description_paragraph: None,
                            })
                            .description_paragraph
                            .get_or_insert_with(|| vec![String::new()]);
                        if position.1 < line.len {
                            position.1 = line_position;
                        }
                        *sub_position = Some(CommentaryPosition::Description(0, 0));
                    }
                }

                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('D'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        position,
                        sub_position,
                        current,
                        command_buffer,
                        ..
                    },
                ) => {
                    log::info!("D");
                    if command_buffer == " " {
                        command_buffer.clear();
                        let line = &mut current.text[position.0];
                        let line_position = get_line_position(position, line, false);
                        // make sure its not inside a word boundary
                        line.commentary
                            .entry(line_position)
                            .or_insert(Commentary {
                                sentence_translation: None,
                                description_paragraph: None,
                            })
                            .description_paragraph
                            .get_or_insert_default();
                        // only update the position if its less than line length
                        if position.1 < line.len {
                            position.1 = line_position;
                        }
                        *sub_position = Some(CommentaryPosition::Description(0, 0));
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc, ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        sub_position,
                        command_buffer,
                        ..
                    },
                ) => {
                    log::info!("esc");
                    if command_buffer == " " {
                        command_buffer.clear();
                        *sub_position = None;
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc, ..
                    }),
                    AppStateKind::Translating {
                        translation_state: translation_state @ TranslationState::Editing,
                        position: _,
                        current: _,
                        ..
                    },
                ) => *translation_state = TranslationState::Normal,
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(char),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Editing,
                        sub_position: Some(CommentaryPosition::Translation(i)),
                        position,
                        current,
                        ..
                    },
                ) => {
                    if let Some(translation) = &mut current.text[position.0]
                        .commentary
                        .get_mut(&(position.1))
                        .unwrap()
                        .sentence_translation
                    {
                        let len = translation.len();
                        // editing is considered setting a new cursor position
                        *i = (*i).min(len.saturating_sub(1));
                        translation.insert(
                            if translation.is_empty() {
                                0
                            } else {
                                char_index_to_byte(*i + 1, translation)
                            },
                            char,
                        );
                        if len != 0 {
                            *i += 1;
                        }
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(char),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Editing,
                        sub_position: Some(CommentaryPosition::Description(line, column)),
                        position,
                        current,
                        ..
                    },
                ) => {
                    if let Some(description) = &mut current.text[position.0]
                        .get_commentary_mut(position.1)
                        .unwrap()
                        .description_paragraph
                    {
                        let line = &mut description[*line];
                        let len = line.len();
                        // editing is considered setting a new cursor position
                        *column = (*column).min(len.saturating_sub(1));
                        line.insert(
                            if line.is_empty() {
                                0
                            } else {
                                char_index_to_byte(*column + 1, line)
                            },
                            char,
                        );
                        if len != 0 {
                            *column += 1;
                        }
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Editing,
                        sub_position: Some(CommentaryPosition::Description(line, column)),
                        position,
                        current,
                        ..
                    },
                ) => {
                    if let Some(description) = &mut current.text[position.0]
                        .get_commentary_mut(position.1)
                        .unwrap()
                        .description_paragraph
                    {
                        description.push(String::new());
                        *line += 1;
                        // editing is considered setting a new cursor position
                        *column = 0;
                    }
                }
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('i'),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: translation_state @ TranslationState::Normal,
                        sub_position: Some(_),
                        ..
                    },
                ) => *translation_state = TranslationState::Editing,
                (
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(' '),
                        ..
                    }),
                    AppStateKind::Translating {
                        translation_state: TranslationState::Normal,
                        command_buffer,
                        ..
                    },
                ) => {
                    command_buffer.push(' ');
                }
                (event, _) => {
                    file_explorer.handle(&event)?;
                    if let Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }) = event
                        && !file_explorer.current().is_dir
                    {
                        println!("\x1b[8l\x1b[1 k");
                        let file = read_to_string(file_explorer.current().path.clone()).unwrap();

                        let current = parse(&file);
                        // turn of any bidi mode
                        app.kind = AppStateKind::Translating {
                            sub_position: None,
                            position: (0, 0),
                            current,
                            command_buffer: String::new(),
                            translation_state: TranslationState::Normal,
                        };
                    }
                }
            }
        }
    }
}

fn get_line_position(position: &(usize, usize), line: &structure::Line, end: bool) -> usize {
    line.words.get_key_value(&position.1).map_or_else(
        || position_or_text_len(position.1, line),
        |(range, _)| {
            if end { range.end } else { range.start }
        },
    )
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
                translation_state,
                position,
                sub_position,
                ..
            } => {
                let text = current
                    .text
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(i, line)| {
                        // Current behaviour is to hide any commentary not current line maybe have
                        // toggle to control
                        if i == position.0 {
                            let column = current
                                .text
                                .get(position.0)
                                .map_or(0, |text| position_or_text_len(position.1, text));

                            let (text, mut plain_text, prev_i) =
                                line.commentary.iter().sorted_by_key(|x| x.0).fold(
                                    (String::new(), line.text, 0),
                                    |(text, mut plain_text, prev_i), (i, commentary)| {
                                        let processing_text = plain_text.split_off(
                                            plain_text.char_indices().nth(*i - prev_i).unwrap().0,
                                        );
                                        let column_cursor =
                                            if position.1 < *i && sub_position.is_none() {
                                                // this is buggy
                                                let column = column - prev_i;
                                                // if plain_text is empty, it len() will be 0, and column +
                                                // 1 will be 1
                                                Some(column)
                                            } else {
                                                None
                                            };
                                        plain_text = bidi_hebrew(&plain_text, column_cursor);
                                        let (translation, description) = if *i == position.1
                                            && let Some(sub_position) = sub_position
                                        {
                                            match sub_position {
                                                CommentaryPosition::Description(line, column) => {
                                                    cursor_ify_description(
                                                        translation_state,
                                                        commentary,
                                                        *line,
                                                        *column,
                                                    )
                                                }
                                                CommentaryPosition::Translation(column) => {
                                                    cursor_ify_translation(
                                                        translation_state,
                                                        commentary,
                                                        *column,
                                                    )
                                                }
                                            }
                                        } else {
                                            (
                                                commentary
                                                    .sentence_translation
                                                    .as_ref()
                                                    .map(|t| bidi_english(t, None)),
                                                commentary.description_paragraph.as_ref().map_or(
                                                    String::new(),
                                                    |text| {
                                                        format!(
                                                            "\n{}\n",
                                                            text.iter()
                                                                .map(|t| bidi_english(t, None))
                                                                .join("\n")
                                                        )
                                                    },
                                                ),
                                            )
                                        };

                                        (
                                            format!(
                                                "{}{}{}{}{}",
                                                text,
                                                if text.is_empty() { "" } else { "\n" },
                                                translation.map_or(String::new(), |s| { s + " " }),
                                                plain_text,
                                                description
                                            ),
                                            processing_text,
                                            *i,
                                        )
                                    },
                                );
                            // sub postion should not be some here technically as its the last thing
                            // on the line
                            let column_cursor = if position.1 >= prev_i && sub_position.is_none() {
                                let column = column - prev_i;
                                Some(column)
                            } else {
                                None
                            };
                            plain_text = bidi_hebrew(&plain_text, column_cursor);
                            let separator = if text.is_empty() && !plain_text.is_empty() {
                                ""
                            } else {
                                "\n"
                            };
                            text + separator + &plain_text
                        } else {
                            bidi_hebrew(&line.text, None)
                        }
                    })
                    .join("\n");
                frame.render_widget(
                    Paragraph::new(text.to_text().unwrap()).centered(),
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

fn bidi_hebrew(plain_text: &str, cursor: Option<usize>) -> String {
    let chars = plain_text.chars();
    if let Some(cursor) = cursor {
        let mut cursor_placed = false;
        bidi_inner_hebrew(
            chars.enumerate(),
            |(_, char)| *char,
            |i| {
                let mut s: String = i
                    .flat_map(|(i, char)| {
                        if i == cursor {
                            cursor_placed = true;
                            vec![
                                '\x1b', '[', '4', '7', ';', '5', 'm', char, '\x1b', '[', '4', '7',
                                ';', '0', 'm',
                            ]
                        } else {
                            vec![char]
                        }
                    })
                    .collect();
                if !cursor_placed {
                    s.insert_str(0, "\x1b[47;5m \x1b[47;0m");
                }
                s
            },
        )
    } else {
        bidi_inner_hebrew(chars, |char| *char, |i| i.collect())
    }
}

fn bidi_inner_hebrew<T, U>(
    plain_text: impl Iterator<Item = T>,
    to_char: impl Fn(&T) -> char + Copy,
    k: impl FnOnce(&mut dyn Iterator<Item = T>) -> U,
) -> U {
    enum CharType {
        Ltr,
        Rtl(char),
    }

    impl PartialEq for CharType {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::Rtl(_l0), Self::Rtl(_r0)) => false,
                _ => core::mem::discriminant(self) == core::mem::discriminant(other),
            }
        }
    }
    k(&mut plain_text
        .chunk_by(move |t| {
            if to_char(t).is_ascii_alphanumeric() {
                CharType::Ltr
            } else {
                CharType::Rtl(to_char(t))
            }
        })
        .into_iter()
        .collect_vec()
        .into_iter()
        .rev()
        .flat_map(|(_, chunk)| chunk))
}

fn position_or_text_len(position: usize, text: &impl CharLength) -> usize {
    (text.char_len().saturating_sub(1)).min(position)
}

fn cursor_ify_description(
    translation_state: &TranslationState,
    commentary: &Commentary,
    line: usize,
    column: usize,
) -> (Option<String>, String) {
    (
        commentary
            .sentence_translation
            .as_ref()
            .map(|t| bidi_english(t, None))
            ,
        commentary
            .description_paragraph
            .as_ref()
            .map_or(String::new(), |text| {
                format!(
                    "\n{}\n",
                    text.iter()
                        .cloned()
                        .enumerate()
                        .map(|(i, s)| {
                            bidi_english(&s, {
                                (i == line).then_some((
                                    column,
                                    *translation_state == TranslationState::Editing,
                                ))
                            })
                        })
                        .join("\n")
                )
            }),
    )
}

fn cursor_ify_translation(
    translation_state: &TranslationState,
    commentary: &Commentary,
    column: usize,
) -> (Option<String>, String) {
    (
        commentary.sentence_translation.as_ref().map(|translation| {
            bidi_english(
                translation,
                Some((column, *translation_state == TranslationState::Editing)),
            )
        }),
        commentary
            .description_paragraph
            .as_ref()
            .map_or(String::new(), |text| {
                format!(
                    "\n{}\n",
                    text.iter().map(|t| bidi_english(t, None)).join("\n")
                )
            }),
    )
}

fn char_index_to_byte(index: usize, text: &str) -> usize {
    text.char_indices()
        .nth(index)
        .map_or(text.len(), |(i, _)| i)
}
fn bidi_english(plain_text: &str, cursor: Option<(usize, bool)>) -> String {
    let chars = plain_text.chars();
    if let Some((column, edit)) = cursor {
        let column = position_or_text_len(column, &plain_text);
        if edit {
            // having cursor after postion is harder with bidi
            // column += 1;
        }

        let mut cursor_placed = false;
        let mut s: String = bidi_inner_english(
            chars.enumerate(),
            |(_, char)| *char,
            |i| {
                i.flat_map(|(i, char)| {
                    if i == column {
                        cursor_placed = true;
                        vec![
                            '\x1b', '[', '4', '7', ';', '5', 'm', char, '\x1b', '[', '4', '7', ';',
                            '0', 'm',
                        ]
                    } else {
                        vec![char]
                    }
                })
                .collect()
            },
        );
        if !cursor_placed {
            s.push_str("\x1b[47;5m \x1b[47;0m");
        }
        s
    } else {
        bidi_inner_english(chars, |char| *char, |i| i.collect())
    }
}

fn bidi_inner_english<T, U>(
    plain_text: impl Iterator<Item = T>,
    to_char: impl Fn(&T) -> char,
    k: impl FnOnce(&mut dyn Iterator<Item = T>) -> U,
) -> U {
    enum CharType {
        Ltr(char),
        Rtl,
    }

    impl PartialEq for CharType {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::Ltr(_l0), Self::Ltr(_r0)) => false,
                _ => core::mem::discriminant(self) == core::mem::discriminant(other),
            }
        }
    }
    k(&mut plain_text
        .chunk_by(|char| {
            let char = to_char(char);
            if char.is_ascii_alphanumeric()
                || char.is_ascii_whitespace()
                || char.is_ascii_punctuation()
            {
                CharType::Ltr(char)
            } else {
                CharType::Rtl
            }
        })
        .into_iter()
        .flat_map(|chunk| chunk.1.collect_vec().into_iter().rev()))
}
