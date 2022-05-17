use crate::config::{Keys, Termusic};
use crate::ui::{DBMsg, Id, Model, Msg};
// use anyhow::Result;
// use rand::seq::SliceRandom;
// use rand::thread_rng;
// use std::collections::VecDeque;
// use std::fs::File;
// use std::io::{BufRead, BufReader, Write};
// use std::path::{Path, PathBuf};
// use std::thread;
// use std::time::Duration;
use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{Alignment, BorderType, TableBuilder, TextSpan};
use tuirealm::{
    event::{Key, KeyEvent, NoUserEvent},
    Component, Event, MockComponent, State, StateValue,
};

use tuirealm::props::{Borders, Color};

#[derive(MockComponent)]
pub struct DBListCriteria {
    component: List,
    keys: Keys,
}

impl DBListCriteria {
    pub fn new(config: &Termusic) -> Self {
        Self {
            component: List::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .title("DataBase", Alignment::Left)
                .scroll(true)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Artist"))
                        .add_row()
                        .add_col(TextSpan::from("Album"))
                        .add_row()
                        .add_col(TextSpan::from("Genre"))
                        .build(),
                ),
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for DBListCriteria {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::DataBase(DBMsg::SearchResult(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::DataBase(DBMsg::CriteriaBlur))
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct DBListSearchResult {
    component: List,
    keys: Keys,
}

impl DBListSearchResult {
    pub fn new(config: &Termusic) -> Self {
        Self {
            component: List::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .title("Result", Alignment::Left)
                .scroll(true)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Artist"))
                        .add_row()
                        .add_col(TextSpan::from("Album"))
                        .add_row()
                        .add_col(TextSpan::from("Genre"))
                        .build(),
                ),
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for DBListSearchResult {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::DataBase(DBMsg::SearchTrack(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::DataBase(DBMsg::SearchResultBlur))
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct DBListSearchTracks {
    component: List,
    keys: Keys,
}

impl DBListSearchTracks {
    pub fn new(config: &Termusic) -> Self {
        Self {
            component: List::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .title("Tracks", Alignment::Left)
                .scroll(true)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Artist"))
                        .add_row()
                        .add_col(TextSpan::from("Album"))
                        .add_row()
                        .add_col(TextSpan::from("Genre"))
                        .build(),
                ),
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for DBListSearchTracks {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::DataBase(DBMsg::SearchTracksBlur))
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Model {
    pub fn database_sync_tracks(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.db_search_tracks.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            table
                .add_col(TextSpan::from(format!("{}", idx + 1)))
                .add_col(TextSpan::from(" "))
                .add_col(TextSpan::from(record.name.to_string()));
        }
        if self.db_search_results.is_empty() {
            table.add_col(TextSpan::from("empty results"));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::DBListSearchTracks,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();

        // self.playlist_update_title();
    }
    pub fn database_sync_results(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.db_search_results.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            table
                .add_col(TextSpan::from(format!("{}", idx + 1)))
                .add_col(TextSpan::from(" "))
                .add_col(TextSpan::from(record));
        }
        if self.db_search_results.is_empty() {
            table.add_col(TextSpan::from("empty results"));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::DBListSearchResult,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();

        // self.playlist_update_title();
    }
    // pub fn playlist_reload(&mut self) {
    //     // keep focus
    //     let mut focus_playlist = false;
    //     if let Ok(f) = self.app.query(&Id::Playlist, Attribute::Focus) {
    //         if Some(AttrValue::Flag(true)) == f {
    //             focus_playlist = true;
    //         }
    //     }

    //     let mut focus_library = false;
    //     if let Ok(f) = self.app.query(&Id::Library, Attribute::Focus) {
    //         if Some(AttrValue::Flag(true)) == f {
    //             focus_library = true;
    //         }
    //     }

    //     assert!(self.app.umount(&Id::Playlist).is_ok());
    //     assert!(self
    //         .app
    //         .mount(
    //             Id::Playlist,
    //             Box::new(Playlist::new(
    //                 &self.config.style_color_symbol,
    //                 &self.config.keys
    //             )),
    //             Vec::new()
    //         )
    //         .is_ok());
    //     self.playlist_sync();
    //     if focus_playlist {
    //         assert!(self.app.active(&Id::Playlist).is_ok());
    //         return;
    //     }

    //     if focus_library {
    //         return;
    //         // assert!(self.app.active(&Id::Library).is_ok());
    //     }

    //     assert!(self.app.active(&Id::Library).is_ok());
    // }

    // fn playlist_filetype_supported(current_node: &str) -> bool {
    //     let p = Path::new(current_node);

    //     #[cfg(any(feature = "mpv", feature = "gst"))]
    //     if let Some(ext) = p.extension() {
    //         if ext == "opus" {
    //             return true;
    //         }
    //         if ext == "aiff" {
    //             return true;
    //         }
    //         if ext == "webm" {
    //             return true;
    //         }
    //     }

    //     match p.extension() {
    //         Some(ext) if ext == "mp3" => true,
    //         // Some(ext) if ext == "aiff" => true,
    //         Some(ext) if ext == "flac" => true,
    //         Some(ext) if ext == "m4a" => true,
    //         // Some(ext) if ext == "opus" => true,
    //         Some(ext) if ext == "ogg" => true,
    //         Some(ext) if ext == "wav" => true,
    //         // Some(ext) if ext == "webm" => true,
    //         Some(_) | None => false,
    //     }
    // }

    // fn playlist_is_playlist(current_node: &str) -> bool {
    //     let p = Path::new(current_node);

    //     match p.extension() {
    //         Some(ext) if ext == "m3u" => true,
    //         Some(ext) if ext == "m3u8" => true,
    //         Some(ext) if ext == "pls" => true,
    //         Some(ext) if ext == "asx" => true,
    //         Some(ext) if ext == "xspf" => true,
    //         Some(_) | None => false,
    //     }
    // }

    // fn playlist_add_playlist(
    //     &mut self,
    //     current_node: &str,
    //     add_playlist_front: bool,
    // ) -> Result<()> {
    //     let p = Path::new(current_node);
    //     if let Some(p_base) = p.parent() {
    //         if let Ok(str) = std::fs::read_to_string(p) {
    //             if let Ok(items) = crate::playlist::decode(&str) {
    //                 let mut index = 0;
    //                 for item in items {
    //                     if !Self::playlist_filetype_supported(&item) {
    //                         continue;
    //                     }
    //                     let url_decoded = urlencoding::decode(&item)?.into_owned();
    //                     let mut url = url_decoded.clone();
    //                     let mut pathbuf = PathBuf::from(p_base);
    //                     if url_decoded.starts_with("http") {
    //                         continue;
    //                     }
    //                     if url_decoded.starts_with("file") {
    //                         url = url_decoded.replace("file://", "");
    //                     }
    //                     let path = Path::new(&url);
    //                     if path.is_relative() {
    //                         pathbuf.push(url);
    //                     } else {
    //                         pathbuf = PathBuf::from(url);
    //                     }

    //                     if add_playlist_front {
    //                         match Track::read_from_path(pathbuf.as_path()) {
    //                             Ok(item) => {
    //                                 self.playlist_items.insert(index, item);
    //                                 index += 1;
    //                             }
    //                             Err(_e) => {
    //                                 index -= 1;
    //                             }
    //                         }
    //                         continue;
    //                     }
    //                     self.playlist_add_item(&pathbuf.to_string_lossy(), false)
    //                         .ok();
    //                 }
    //             }
    //         }
    //     }
    //     self.playlist_sync();
    //     Ok(())
    // }

    // fn playlist_add_item(&mut self, current_node: &str, add_playlist_front: bool) -> Result<()> {
    //     if Self::playlist_is_playlist(current_node) {
    //         self.playlist_add_playlist(current_node, add_playlist_front)?;
    //         return Ok(());
    //     }
    //     if !Self::playlist_filetype_supported(current_node) {
    //         return Ok(());
    //     }
    //     match Track::read_from_path(current_node) {
    //         Ok(item) => {
    //             if add_playlist_front {
    //                 self.playlist_items.push_front(item);
    //             } else {
    //                 self.playlist_items.push_back(item);
    //             }
    //             self.playlist_sync();
    //         }
    //         Err(e) => return Err(e),
    //     }
    //     Ok(())
    // }
    // fn playlist_add_items(&mut self, p: &Path) {
    //     let new_items = Self::library_dir_children(p);
    //     let mut index = 0;
    //     for s in &new_items {
    //         if self.config.add_playlist_front {
    //             if !Self::playlist_filetype_supported(s) {
    //                 continue;
    //             }
    //             match Track::read_from_path(s) {
    //                 Ok(item) => {
    //                     self.playlist_items.insert(index, item);
    //                     index += 1;
    //                 }
    //                 Err(_e) => {
    //                     index -= 1;
    //                 }
    //             }
    //             continue;
    //         }

    //         self.playlist_add_item(s, false).ok();
    //     }
    //     self.playlist_sync();
    // }
    // pub fn playlist_add(&mut self, current_node: &str) {
    //     let p: &Path = Path::new(&current_node);
    //     if !p.exists() {
    //         return;
    //     }

    //     if p.is_dir() {
    //         self.playlist_add_items(p);
    //     } else if let Err(e) = self.playlist_add_item(current_node, self.config.add_playlist_front)
    //     {
    //         self.mount_error_popup(format!("Add Playlist error: {}", e).as_str());
    //     }
    // }

    // pub fn playlist_sync(&mut self) {
    //     let mut table: TableBuilder = TableBuilder::default();

    //     for (idx, record) in self.playlist_items.iter().enumerate() {
    //         if idx > 0 {
    //             table.add_row();
    //         }

    //         let duration = record.duration_formatted().to_string();
    //         let duration_string = format!("[{:^7.7}]", duration);

    //         let noname_string = "No Name".to_string();
    //         let name = record.name().unwrap_or(&noname_string);
    //         let artist = record.artist().unwrap_or(name);
    //         let title = record.title().unwrap_or("Unknown Title");

    //         table
    //             .add_col(TextSpan::new(duration_string.as_str()))
    //             .add_col(TextSpan::new(artist).fg(tuirealm::tui::style::Color::LightYellow))
    //             .add_col(TextSpan::new(title).bold())
    //             .add_col(TextSpan::new(record.album().unwrap_or("Unknown Album")));
    //     }
    //     if self.playlist_items.is_empty() {
    //         table.add_col(TextSpan::from("0"));
    //         table.add_col(TextSpan::from("empty playlist"));
    //         table.add_col(TextSpan::from(""));
    //         table.add_col(TextSpan::from(""));
    //     }

    //     let table = table.build();
    //     self.app
    //         .attr(
    //             &Id::Playlist,
    //             tuirealm::Attribute::Content,
    //             tuirealm::AttrValue::Table(table),
    //         )
    //         .ok();

    //     self.playlist_update_title();
    // }
    // pub fn playlist_delete_item(&mut self, index: usize) {
    //     if self.playlist_items.is_empty() {}
    //     self.playlist_items.remove(index);
    //     self.playlist_sync();
    // }

    // pub fn playlist_empty(&mut self) {
    //     self.playlist_items.clear();
    //     self.playlist_sync();
    //     // self.app.active(&Id::Library).ok();
    // }

    // pub fn playlist_save(&mut self) -> Result<()> {
    //     let mut path = get_app_config_path()?;
    //     path.push("playlist.log");
    //     let mut file = File::create(path.as_path())?;
    //     for i in &self.playlist_items {
    //         if let Some(f) = i.file() {
    //             writeln!(&mut file, "{}", f)?;
    //         }
    //     }

    //     Ok(())
    // }

    // pub fn playlist_load(&mut self) -> Result<()> {
    //     let mut path = get_app_config_path()?;
    //     path.push("playlist.log");

    //     let file = if let Ok(f) = File::open(path.as_path()) {
    //         f
    //     } else {
    //         File::create(path.as_path())?;
    //         File::open(path)?
    //     };
    //     let reader = BufReader::new(file);
    //     let lines: Vec<_> = reader
    //         .lines()
    //         .map(|line| line.unwrap_or_else(|_| "Error".to_string()))
    //         .collect();

    //     let tx = self.sender_playlist_items.clone();

    //     thread::spawn(move || {
    //         let mut playlist_items = VecDeque::new();
    //         for line in &lines {
    //             if let Ok(s) = Track::read_from_path(line) {
    //                 playlist_items.push_back(s);
    //             };
    //         }
    //         tx.send(playlist_items).ok();
    //     });

    //     // let mut playlist_items = VecDeque::new();
    //     // for line in &lines {
    //     //     if let Ok(s) = Song::from_str(line) {
    //     //         playlist_items.push_back(s);
    //     //     };
    //     // }

    //     // self.playlist_items = playlist_items;
    //     Ok(())
    // }

    // pub fn playlist_shuffle(&mut self) {
    //     let mut rng = thread_rng();
    //     self.playlist_items.make_contiguous().shuffle(&mut rng);
    //     self.playlist_sync();
    // }

    // pub fn playlist_update_library_delete(&mut self) {
    //     self.playlist_items.retain(|x| {
    //         x.file().map_or(false, |p| {
    //             let path = Path::new(p);
    //             path.exists()
    //         })
    //     });

    //     self.playlist_sync();
    //     // assert!(self.app.active(&Id::Library).is_ok());
    // }
    // pub fn playlist_update_title(&mut self) {
    //     let mut duration = Duration::from_secs(0);
    //     for v in &self.playlist_items {
    //         duration += v.duration();
    //     }
    //     let add_queue = if self.config.add_playlist_front {
    //         if self.config.playlist_display_symbol {
    //             // "\u{1f51d}"
    //             "\u{fb22}"
    //             // "ﬢ"
    //         } else {
    //             "next"
    //         }
    //     } else if self.config.playlist_display_symbol {
    //         "\u{fb20}"
    //         // "ﬠ"
    //     } else {
    //         "last"
    //     };
    //     let title = format!(
    //         "\u{2500} Playlist \u{2500}\u{2500}\u{2524} Total {} tracks | {} | Loop: {} | Add: {} \u{251c}\u{2500}",
    //         self.playlist_items.len(),
    //         Track::duration_formatted_short(&duration),
    //         self.config.loop_mode.display(self.config.playlist_display_symbol),
    //         add_queue
    //     );
    //     self.app
    //         .attr(
    //             &Id::Playlist,
    //             tuirealm::Attribute::Title,
    //             tuirealm::AttrValue::Title((title, Alignment::Left)),
    //         )
    //         .ok();
    // }
    // pub fn playlist_cycle_loop_mode(&mut self) {
    //     match self.config.loop_mode {
    //         Loop::Queue => {
    //             self.config.loop_mode = Loop::Playlist;
    //         }
    //         Loop::Playlist => {
    //             self.config.loop_mode = Loop::Single;
    //             if let Some(song) = self.playlist_items.pop_back() {
    //                 self.playlist_items.push_front(song);
    //             }
    //         }
    //         Loop::Single => {
    //             self.config.loop_mode = Loop::Queue;
    //             if let Some(song) = self.playlist_items.pop_front() {
    //                 self.playlist_items.push_back(song);
    //             }
    //         }
    //     };
    //     self.playlist_sync();
    //     self.playlist_update_title();
    // }
    // pub fn playlist_play_selected(&mut self, index: usize) {
    //     // self.time_pos = 0;
    //     if let Some(song) = self.playlist_items.remove(index) {
    //         self.playlist_items.push_front(song);
    //         self.playlist_sync();
    //         // self.status = Some(Status::Stopped);
    //         self.player_next();
    //     }
    // }
    // pub fn playlist_update_search(&mut self, input: &str) {
    //     let mut table: TableBuilder = TableBuilder::default();
    //     let mut idx = 0;
    //     let search = format!("*{}*", input.to_lowercase());
    //     for record in &self.playlist_items {
    //         let artist = record.artist().unwrap_or("Unknown artist");
    //         let title = record.title().unwrap_or("Unknown title");
    //         if wildmatch::WildMatch::new(&search).matches(&artist.to_lowercase())
    //             | wildmatch::WildMatch::new(&search).matches(&title.to_lowercase())
    //         {
    //             if idx > 0 {
    //                 table.add_row();
    //             }

    //             let duration = record.duration_formatted().to_string();
    //             let duration_string = format!("[{:^6.6}]", duration);

    //             let noname_string = "No Name".to_string();
    //             let name = record.name().unwrap_or(&noname_string);
    //             let artist = record.artist().unwrap_or(name);
    //             let title = record.title().unwrap_or("Unknown Title");
    //             let file_name = record.file().unwrap_or("no file");

    //             table
    //                 .add_col(TextSpan::new(duration_string.as_str()))
    //                 .add_col(TextSpan::new(artist).fg(tuirealm::tui::style::Color::LightYellow))
    //                 .add_col(TextSpan::new(title).bold())
    //                 .add_col(TextSpan::new(file_name));
    //             // .add_col(TextSpan::new(record.album().unwrap_or("Unknown Album")));
    //             idx += 1;
    //         }
    //     }
    //     if self.playlist_items.is_empty() {
    //         table.add_col(TextSpan::from("0"));
    //         table.add_col(TextSpan::from("empty playlist"));
    //         table.add_col(TextSpan::from(""));
    //     }
    //     let table = table.build();

    //     self.general_search_update_show(table);
    // }

    // pub fn playlist_locate(&mut self, index: usize) {
    //     assert!(self
    //         .app
    //         .attr(
    //             &Id::Playlist,
    //             Attribute::Value,
    //             AttrValue::Payload(PropPayload::One(PropValue::Usize(index))),
    //         )
    //         .is_ok());
    // }

    // pub fn playlist_swap_down(&mut self, index: usize) {
    //     if index < self.playlist_items.len() - 1 {
    //         if let Some(song) = self.playlist_items.remove(index) {
    //             self.playlist_items.insert(index + 1, song);
    //             self.playlist_sync();
    //         }
    //     }
    // }

    // pub fn playlist_swap_up(&mut self, index: usize) {
    //     if index > 0 {
    //         if let Some(song) = self.playlist_items.remove(index) {
    //             self.playlist_items.insert(index - 1, song);
    //             self.playlist_sync();
    //         }
    //     }
    // }
}