#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// #![warn(clippy::restriction)]
/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
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
mod app;
mod config;
mod invidious;
mod player;
mod song;
mod songtag;
mod ui;

use app::App;
use config::Termusic;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    // match args.len() {}

    args.remove(0);
    for i in args.clone() {
        let i = i.as_str();
        match i {
            "-v" => println!("Termusic version is: {}", VERSION),

            "-h" => println!(
                r"Termusic help:
-v print version and exit.
-h print this message.
no arguments: start termusic with ~/.config/termusic/config.toml"
            ),

            _ => println!(
                r"Unknown arguments
Termusic help:
-v print version and exit.
-h print this message.
no arguments: start termusic with ~/.config/termusic/config.toml"
            ),
        }
    }
    if !args.is_empty() {
        return;
    }

    let config = Termusic::load().unwrap_or_default();

    let mut app: App = App::new(config);
    app.run();
}
