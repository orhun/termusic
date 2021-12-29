//! ## Components
//!
//! demo example components

mod general_search;
/**
 * MIT License
 *
 * tui-realm - Copyright (C) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
// -- modules
// mod clock;
// mod counter;
mod label;
mod lyric;
mod music_library;
mod playlist;
mod popups;
mod progress;
// mod table_playlist;
mod color_editor;
mod tag_editor;
mod xywh;
mod youtube_search;

// -- export
// pub use clock::Clock;
// pub use counter::{Digit, Letter};
pub use general_search::{GSInputPopup, GSTablePopup, Source};
pub use label::Label;
pub use lyric::Lyric;
pub use music_library::MusicLibrary;
pub use playlist::Playlist;
pub use popups::{
    DeleteConfirmInputPopup, DeleteConfirmRadioPopup, ErrorPopup, HelpPopup, MessagePopup,
    QuitPopup,
};
pub use progress::Progress;
// pub use table_playlist::Table;
pub use youtube_search::{YSInputPopup, YSTablePopup};
//Tag Edotor Controls
pub use color_editor::{
    load_alacritty_theme, AlacrittyTheme, CEHelpPopup, CELibraryBackground, CELibraryBorder,
    CELibraryForeground, CELibraryHighlight, CELibraryHighlightSymbol, CELibraryTitle,
    CELyricBackground, CELyricBorder, CELyricForeground, CELyricTitle, CEPlaylistBackground,
    CEPlaylistBorder, CEPlaylistForeground, CEPlaylistHighlight, CEPlaylistHighlightSymbol,
    CEPlaylistTitle, CEProgressBackground, CEProgressBorder, CEProgressForeground, CEProgressTitle,
    CERadioOk, CESelectColor, ColorConfig, StyleColorSymbol, ThemeSelectTable,
};
pub use tag_editor::{
    TECounterDelete, TEHelpPopup, TEInputArtist, TEInputTitle, TERadioTag, TESelectLyric,
    TETableLyricOptions, TETextareaLyric,
};
pub use xywh::Xywh;

use crate::config::Termusic;
use crate::ui::{CEMsg, GSMsg, Id, Loop, Model, Msg, PLMsg, Status, YSMsg};
use tui_realm_stdlib::Phantom;
use tuirealm::listener::{ListenerResult, Poll};
use tuirealm::props::{Alignment, Borders, Color, Style};
use tuirealm::tui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::tui::widgets::Block;
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers},
    Component, Event, MockComponent,
};
use tuirealm::{Sub, SubClause, SubEventClause};
#[derive(PartialEq, Clone, PartialOrd)]
pub enum UserEvent {
    QuitApp, // ... other events if you need
}
impl Eq for UserEvent {}

impl Poll<UserEvent> for HotkeyHandler {
    fn poll(&mut self) -> ListenerResult<Option<Event<UserEvent>>> {
        // ... do something ...
        Ok(Some(Event::User(UserEvent::QuitApp)))
    }
}
pub struct HotkeyHandler {}

impl HotkeyHandler {
    pub const fn new() -> Self {
        Self {}
    }
    // ...
}
#[derive(MockComponent)]
pub struct GlobalListener {
    component: Phantom,
    // key_quit: char,
}

impl GlobalListener {
    pub fn new(_config: &Termusic) -> Self {
        Self {
            component: Phantom::default(),
            // key_quit: config.key_quit,
        }
    }
}

impl Component<Msg, UserEvent> for GlobalListener {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        // let key_quit = KeyEvent {
        //     code: Key::Char('q'),
        //     modifiers: KeyModifiers::NONE,
        // };
        match ev {
            Event::WindowResize(..) => Some(Msg::UpdatePhoto),
            Event::Keyboard(KeyEvent {
                code: Key::Esc | Key::Char('q'),
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::QuitPopupShow),
            // Event::Keyboard(key_quit) => Some(Msg::QuitPopupShow),
            // Event::Keyboard(self.keys) => Some(Msg::QuitPopupShow),
            Event::Keyboard(KeyEvent {
                code: Key::Char(' '),
                ..
            }) => Some(Msg::PlayerTogglePause),
            Event::Keyboard(KeyEvent {
                code: Key::Char('n'),
                ..
            }) => Some(Msg::Playlist(PLMsg::NextSong)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('N'),
                modifiers: KeyModifiers::SHIFT,
            }) => Some(Msg::Playlist(PLMsg::PrevSong)),
            Event::Keyboard(
                KeyEvent {
                    code: Key::Char('-'),
                    ..
                }
                | KeyEvent {
                    code: Key::Char('_'),
                    modifiers: KeyModifiers::SHIFT,
                },
            ) => Some(Msg::PlayerVolumeDown),
            Event::Keyboard(
                KeyEvent {
                    code: Key::Char('='),
                    ..
                }
                | KeyEvent {
                    code: Key::Char('+'),
                    modifiers: KeyModifiers::SHIFT,
                },
            ) => Some(Msg::PlayerVolumeUp),
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::HelpPopupShow),

            Event::Keyboard(KeyEvent {
                code: Key::Char('f'),
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::PlayerSeek(5)),

            Event::Keyboard(KeyEvent {
                code: Key::Char('b'),
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::PlayerSeek(-5)),

            Event::Keyboard(KeyEvent {
                code: Key::Char('F'),
                modifiers: KeyModifiers::SHIFT,
            }) => Some(Msg::LyricAdjustDelay(1000)),

            Event::Keyboard(KeyEvent {
                code: Key::Char('B'),
                modifiers: KeyModifiers::SHIFT,
            }) => Some(Msg::LyricAdjustDelay(-1000)),

            Event::Keyboard(KeyEvent {
                code: Key::Char('T'),
                modifiers: KeyModifiers::SHIFT,
            }) => Some(Msg::LyricCycle),

            Event::Keyboard(KeyEvent {
                code: Key::Char('C'),
                modifiers: KeyModifiers::SHIFT,
            }) => Some(Msg::ColorEditor(CEMsg::ColorEditorShow)),

            _ => None,
        }
    }
}

impl Model {
    /// global listener subscriptions
    #[allow(clippy::too_many_lines)]
    pub fn subscribe() -> Vec<Sub<Id, UserEvent>> {
        vec![
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Esc,
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('q'),
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char(' '),
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('n'),
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('N'),
                    modifiers: KeyModifiers::SHIFT,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('-'),
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('='),
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('_'),
                    modifiers: KeyModifiers::SHIFT,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('+'),
                    modifiers: KeyModifiers::SHIFT,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('h'),
                    modifiers: KeyModifiers::CONTROL,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('f'),
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('b'),
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('F'),
                    modifiers: KeyModifiers::SHIFT,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('B'),
                    modifiers: KeyModifiers::SHIFT,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('T'),
                    modifiers: KeyModifiers::SHIFT,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('C'),
                    modifiers: KeyModifiers::SHIFT,
                }),
                SubClause::Always,
            ),
            Sub::new(SubEventClause::WindowResize, SubClause::Always),
        ]
    }
    pub fn player_next(&mut self) {
        if self.playlist_items.is_empty() {
            return;
        }
        self.time_pos = 0;
        self.status = Some(Status::Running);
        if let Some(song) = self.playlist_items.pop_front() {
            if let Some(file) = song.file() {
                self.player.add_and_play(file);
            }
            match self.config.loop_mode {
                Loop::Playlist => self.playlist_items.push_back(song.clone()),
                Loop::Single => self.playlist_items.push_front(song.clone()),
                Loop::Queue => {}
            }
            self.playlist_sync();
            self.current_song = Some(song);
            if let Err(e) = self.update_photo() {
                self.mount_error_popup(format!("update photo error: {}", e).as_str());
            };
            self.progress_update_title();
            self.update_playing_song();
        }
    }

    pub fn player_previous(&mut self) {
        if let Loop::Single | Loop::Queue = self.config.loop_mode {
            return;
        }

        if self.playlist_items.is_empty() {
            return;
        }

        if let Some(song) = self.playlist_items.pop_back() {
            self.playlist_items.push_front(song);
        }
        if let Some(song) = self.playlist_items.pop_back() {
            self.playlist_items.push_front(song);
        }
        self.player_next();
    }

    pub fn player_toggle_pause(&mut self) {
        if self.player.is_paused() {
            self.status = Some(Status::Running);
            self.player.resume();
        } else {
            self.status = Some(Status::Paused);
            self.player.pause();
        }
    }

    pub fn player_seek(&mut self, offset: i64) {
        self.player.seek(offset).ok();
        self.progress_update();
    }
}
///
/// Get block
pub fn get_block<'a>(props: &Borders, title: (String, Alignment), focus: bool) -> Block<'a> {
    Block::default()
        .borders(props.sides)
        .border_style(if focus {
            props.style()
        } else {
            Style::default().fg(Color::Reset).bg(Color::Reset)
        })
        .border_type(props.modifiers)
        .title(title.0)
        .title_alignment(title.1)
}

// Draw an area (WxH / 3) in the middle of the parent area
pub fn draw_area_in(parent: Rect, width: u16, height: u16) -> Rect {
    let new_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height) / 2),
                Constraint::Percentage(height),
                Constraint::Percentage((100 - height) / 2),
            ]
            .as_ref(),
        )
        .split(parent);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - width) / 2),
                Constraint::Percentage(width),
                Constraint::Percentage((100 - width) / 2),
            ]
            .as_ref(),
        )
        .split(new_area[1])[1]
}

pub fn draw_area_top_right(parent: Rect, width: u16, height: u16) -> Rect {
    let new_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(3),
                Constraint::Percentage(height),
                Constraint::Percentage(100 - 3 - height),
            ]
            .as_ref(),
        )
        .split(parent);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(100 - 1 - width),
                Constraint::Percentage(width),
                Constraint::Percentage(1),
            ]
            .as_ref(),
        )
        .split(new_area[1])[1]
}

#[cfg(test)]
mod tests {

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_utils_ui_draw_area_in() {
        let area: Rect = Rect::new(0, 0, 1024, 512);
        let child: Rect = draw_area_in(area, 75, 30);
        assert_eq!(child.x, 43);
        assert_eq!(child.y, 63);
        assert_eq!(child.width, 271);
        assert_eq!(child.height, 54);
    }
}
