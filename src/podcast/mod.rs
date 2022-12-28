// Thanks to the author of shellcaster(https://github.com/jeff-hughes/shellcaster). Most parts of following code are taken from it.

pub mod db;

use crate::ui::{Msg, PCMsg};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use db::Database;
use lazy_static::lazy_static;
use opml::{Body, Head, Outline, OPML};
use regex::{Match, Regex};
use rfc822_sanitizer::parse_from_rfc2822_with_fallback;
use rss::{Channel, Item};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

lazy_static! {
    /// Regex for parsing an episode "duration", which could take the form
    /// of HH:MM:SS, MM:SS, or SS.
    static ref RE_DURATION: Regex = Regex::new(r"(\d+)(?::(\d+))?(?::(\d+))?").expect("Regex error");
}

/// Struct holding data about an individual podcast feed. This includes a
/// (possibly empty) vector of episodes.
#[derive(Debug, Clone)]
pub struct Podcast {
    pub id: i64,
    pub title: String,
    pub sort_title: String,
    pub url: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub explicit: Option<bool>,
    pub last_checked: DateTime<Utc>,
    pub episodes: Vec<Episode>,
}

impl Podcast {
    // Counts and returns the number of unplayed episodes in the podcast.
    // fn num_unplayed(&self) -> usize {
    //     return self
    //         .episodes
    //         .iter()
    //         .map(|ep| !ep.is_played() as usize)
    //         .iter()
    //         .sum();
    // }
}

/// Struct holding data about an individual podcast episode. Most of this
/// is metadata, but if the episode has been downloaded to the local
/// machine, the filepath will be included here as well. `played`
/// indicates whether the podcast has been marked as played or unplayed.
#[derive(Debug, Clone)]
pub struct Episode {
    pub id: i64,
    pub pod_id: i64,
    pub title: String,
    pub url: String,
    pub guid: String,
    pub description: String,
    pub pubdate: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub path: Option<PathBuf>,
    pub played: bool,
}

impl Episode {
    /// Formats the duration in seconds into an HH:MM:SS format.
    pub fn format_duration(&self) -> String {
        return match self.duration {
            Some(dur) => {
                let mut seconds = dur;
                let hours = seconds / 3600;
                seconds -= hours * 3600;
                let minutes = seconds / 60;
                seconds -= minutes * 60;
                format!("{hours:02}:{minutes:02}:{seconds:02}")
            }
            None => "--:--:--".to_string(),
        };
    }
}

/// Struct holding data about an individual podcast feed, before it has
/// been inserted into the database. This includes a
/// (possibly empty) vector of episodes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PodcastNoId {
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub explicit: Option<bool>,
    pub last_checked: DateTime<Utc>,
    pub episodes: Vec<EpisodeNoId>,
}

/// Struct holding data about an individual podcast episode, before it
/// has been inserted into the database.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpisodeNoId {
    pub title: String,
    pub url: String,
    pub guid: String,
    pub description: String,
    pub pubdate: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
}

/// Struct holding data about an individual podcast episode, specifically
/// for the popup window that asks users which new episodes they wish to
/// download.
#[derive(Debug, Clone)]
pub struct NewEpisode {
    pub id: i64,
    pub pod_id: i64,
    pub title: String,
    pub pod_title: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PodcastFeed {
    pub id: Option<i64>,
    pub url: String,
    pub title: Option<String>,
}

impl PodcastFeed {
    pub fn new(id: Option<i64>, url: String, title: Option<String>) -> Self {
        return Self { id, url, title };
    }
}

pub fn podcast_import(xml: &str) -> Result<Vec<PodcastFeed>> {
    return match OPML::from_str(xml) {
        Err(err) => Err(anyhow!(err)),
        Ok(opml) => {
            let mut feeds = Vec::new();
            for pod in opml.body.outlines.into_iter() {
                if pod.xml_url.is_some() {
                    // match against title attribute first -- if this is
                    // not set or empty, then match against the text
                    // attribute; this must be set, but can be empty
                    let temp_title = pod.title.filter(|t| !t.is_empty());
                    let title = match temp_title {
                        Some(t) => Some(t),
                        None => {
                            if pod.text.is_empty() {
                                None
                            } else {
                                Some(pod.text)
                            }
                        }
                    };
                    feeds.push(PodcastFeed::new(None, pod.xml_url.unwrap(), title));
                }
            }
            Ok(feeds)
        }
    };
}

/// Converts the current set of podcast feeds to the OPML format
pub fn podcast_export(podcasts: Vec<Podcast>) -> OPML {
    let date = Utc::now();
    let mut opml = OPML {
        head: Some(Head {
            title: Some("Shellcaster Podcast Feeds".to_string()),
            date_created: Some(date.to_rfc2822()),
            ..Head::default()
        }),
        ..Default::default()
    };

    let mut outlines = Vec::new();

    for pod in podcasts.iter() {
        // opml.add_feed(&pod.title, &pod.url);
        outlines.push(Outline {
            text: pod.title.clone(),
            r#type: Some("rss".to_string()),
            xml_url: Some(pod.url.clone()),
            title: Some(pod.title.clone()),
            ..Outline::default()
        });
    }

    opml.body = Body { outlines };
    return opml;
}
/// Spawns a new thread to check a feed and retrieve podcast data.
pub fn check_feed(
    feed: PodcastFeed,
    max_retries: usize,
    threadpool: &Threadpool,
    tx_to_main: mpsc::Sender<Msg>,
) {
    threadpool.execute(move || match get_feed_data(feed.url.clone(), max_retries) {
        Ok(pod) => match feed.id {
            Some(id) => {
                tx_to_main
                    .send(Msg::Podcast(PCMsg::SyncData((id, pod))))
                    .expect("Thread messaging error");
            }
            None => tx_to_main
                .send(Msg::Podcast(PCMsg::NewData(pod)))
                .expect("Thread messaging error"),
        },
        Err(_err) => tx_to_main
            .send(Msg::Podcast(PCMsg::Error(feed)))
            .expect("Thread messaging error"),
    });
}

/// Given a URL, this attempts to pull the data about a podcast and its
/// episodes from an RSS feed.
fn get_feed_data(url: String, mut max_retries: usize) -> Result<PodcastNoId> {
    let agent = ureq::builder()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(20))
        .build();

    let request: Result<ureq::Response> = loop {
        let response = agent.get(&url).call();
        match response {
            Ok(resp) => break Ok(resp),
            Err(_) => {
                max_retries -= 1;
                if max_retries == 0 {
                    break Err(anyhow!("No response from feed"));
                }
            }
        }
    };

    return match request {
        Ok(resp) => {
            let mut reader = resp.into_reader();
            let mut resp_data = Vec::new();
            reader.read_to_end(&mut resp_data)?;

            let channel = Channel::read_from(&resp_data[..])?;
            Ok(parse_feed_data(channel, &url))
        }
        Err(err) => Err(err),
    };
}

/// Given a Channel with the RSS feed data, this parses the data about a
/// podcast and its episodes and returns a Podcast. There are existing
/// specifications for podcast RSS feeds that a feed should adhere to, but
/// this does try to make some attempt to account for the possibility that
/// a feed might not be valid according to the spec.
fn parse_feed_data(channel: Channel, url: &str) -> PodcastNoId {
    let title = channel.title().to_string();
    let url = url.to_string();
    let description = Some(channel.description().to_string());
    let last_checked = Utc::now();

    let mut author = None;
    let mut explicit = None;
    if let Some(itunes) = channel.itunes_ext() {
        author = itunes.author().map(|a| a.to_string());
        explicit = match itunes.explicit() {
            None => None,
            Some(s) => {
                let ss = s.to_lowercase();
                match &ss[..] {
                    "yes" | "explicit" | "true" => Some(true),
                    "no" | "clean" | "false" => Some(false),
                    _ => None,
                }
            }
        };
    }

    let mut episodes = Vec::new();
    let items = channel.into_items();
    if !items.is_empty() {
        for item in &items {
            episodes.push(parse_episode_data(item));
        }
    }

    return PodcastNoId {
        title,
        url,
        description,
        author,
        explicit,
        last_checked,
        episodes,
    };
}

/// For an item (episode) in an RSS feed, this pulls data about the item
/// and converts it to an Episode. There are existing specifications for
/// podcast RSS feeds that a feed should adhere to, but this does try to
/// make some attempt to account for the possibility that a feed might
/// not be valid according to the spec.
fn parse_episode_data(item: &Item) -> EpisodeNoId {
    let title = match item.title() {
        Some(s) => s.to_string(),
        None => "".to_string(),
    };
    let url = match item.enclosure() {
        Some(enc) => enc.url().to_string(),
        None => "".to_string(),
    };
    let guid = match item.guid() {
        Some(guid) => guid.value().to_string(),
        None => "".to_string(),
    };
    let description = match item.description() {
        Some(dsc) => dsc.to_string(),
        None => "".to_string(),
    };
    let pubdate = match item.pub_date() {
        Some(pd) => match parse_from_rfc2822_with_fallback(pd) {
            Ok(date) => {
                // this is a bit ridiculous, but it seems like
                // you have to convert from a DateTime<FixedOffset>
                // to a NaiveDateTime, and then from there create
                // a DateTime<Utc>; see
                // https://github.com/chronotope/chrono/issues/169#issue-239433186
                Some(DateTime::from_utc(date.naive_utc(), Utc))
            }
            Err(_) => None,
        },
        None => None,
    };

    let mut duration = None;
    if let Some(itunes) = item.itunes_ext() {
        duration = duration_to_int(itunes.duration()).map(|dur| dur as i64);
    }

    return EpisodeNoId {
        title,
        url,
        guid,
        description,
        pubdate,
        duration,
    };
}

/// Given a string representing an episode duration, this attempts to
/// convert to an integer representing the duration in seconds. Covers
/// formats HH:MM:SS, MM:SS, and SS. If the duration cannot be converted
/// (covering numerous reasons), it will return None.
fn duration_to_int(duration: Option<&str>) -> Option<i32> {
    match duration {
        Some(dur) => {
            match RE_DURATION.captures(dur) {
                Some(cap) => {
                    /*
                     * Provided that the regex succeeds, we should have
                     * 4 capture groups (with 0th being the full match).
                     * Depending on the string format, however, some of
                     * these may return None. We first loop through the
                     * capture groups and push Some results to an array.
                     * This will fail on the first non-numeric value,
                     * so the duration is parsed only if all components
                     * of it were successfully converted to integers.
                     * Finally, we convert hours, minutes, and seconds
                     * into a total duration in seconds and return.
                     */

                    let mut times = [None; 3];
                    let mut counter = 0;
                    // cap[0] is always full match
                    for c in cap.iter().skip(1).flatten() {
                        if let Ok(intval) = regex_to_int(c) {
                            times[counter] = Some(intval);
                            counter += 1;
                        } else {
                            return None;
                        }
                    }

                    return match counter {
                        // HH:MM:SS
                        3 => Some(
                            times[0].unwrap() * 60 * 60
                                + times[1].unwrap() * 60
                                + times[2].unwrap(),
                        ),
                        // MM:SS
                        2 => Some(times[0].unwrap() * 60 + times[1].unwrap()),
                        // SS
                        1 => times[0],
                        _ => None,
                    };
                }
                None => None,
            }
        }
        None => None,
    }
}

/// Helper function converting a match from a regex capture group into an
/// integer.
fn regex_to_int(re_match: Match) -> Result<i32, std::num::ParseIntError> {
    let mstr = re_match.as_str();
    mstr.parse::<i32>()
}

// Much of the threadpool implementation here was taken directly from
// the Rust Book: https://doc.rust-lang.org/book/ch20-02-multithreaded.html
// and https://doc.rust-lang.org/book/ch20-03-graceful-shutdown-and-cleanup.html

/// Manages a threadpool of a given size, sending jobs to workers as
/// necessary. Implements Drop trait to allow threads to complete
/// their current jobs before being stopped.
pub struct Threadpool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<JobMessage>,
}

impl Threadpool {
    /// Creates a new Threadpool of a given size.
    pub fn new(n_threads: usize) -> Threadpool {
        let (sender, receiver) = mpsc::channel();
        let receiver_lock = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(n_threads);

        for _ in 0..n_threads {
            workers.push(Worker::new(Arc::clone(&receiver_lock)));
        }

        return Threadpool { workers, sender };
    }

    /// Adds a new job to the threadpool, passing closure to first
    /// available worker.
    pub fn execute<F>(&self, func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(func);
        self.sender
            .send(JobMessage::NewJob(job))
            .expect("Thread messaging error");
    }
}

impl Drop for Threadpool {
    /// Upon going out of scope, Threadpool sends terminate message to
    /// all workers but allows them to complete current jobs.
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender
                .send(JobMessage::Terminate)
                .expect("Thread messaging error");
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                // joins to ensure threads finish job before stopping
                thread.join().expect("Error dropping threads");
            }
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Messages used by Threadpool to communicate with Workers.
enum JobMessage {
    NewJob(Job),
    Terminate,
}

/// Used by Threadpool to complete jobs. Each Worker manages a single
/// thread.
struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new Worker, which waits for Jobs to be passed by the
    /// Threadpool.
    fn new(receiver: Arc<Mutex<mpsc::Receiver<JobMessage>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .expect("Threadpool error")
                .recv()
                .expect("Thread messaging error");

            match message {
                JobMessage::NewJob(job) => job(),
                JobMessage::Terminate => break,
            }
        });

        return Worker {
            thread: Some(thread),
        };
    }
}

/// Imports a list of podcasts from OPML format, either reading from a
/// file or from stdin. If the `replace` flag is set, this replaces all
/// existing data in the database.
pub fn import(db_path: &Path, filepath: &str) -> Result<()> {
    // read from file or from stdin
    let mut f =
        File::open(filepath).with_context(|| format!("Could not open OPML file: {filepath}"))?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .with_context(|| format!("Failed to read from OPML file: {filepath}"))?;
    let xml = contents;

    let mut podcast_list = import_opml(xml).with_context(|| {
        "Could not properly parse OPML file -- file may be formatted improperly or corrupted."
    })?;

    if podcast_list.is_empty() {
        println!("No podcasts to import.");
        return Ok(());
    }

    let db_inst = db::Database::connect(db_path)?;

    // delete database if we are replacing the data
    // if args.is_present("replace") {
    //     db_inst
    //         .clear_db()
    //         .with_context(|| "Error clearing database")?;
    // } else {
    let old_podcasts = db_inst.get_podcasts()?;

    // if URL is already in database, remove it from import
    podcast_list = podcast_list
        .into_iter()
        .filter(|pod| {
            for op in &old_podcasts {
                if pod.url == op.url {
                    return false;
                }
            }
            return true;
        })
        .collect();
    // }

    // check again, now that we may have removed feeds after looking at
    // the database
    if podcast_list.is_empty() {
        println!("No podcasts to import.");
        return Ok(());
    }

    println!("Importing {} podcasts...", podcast_list.len());

    // let threadpool = Threadpool::new(config.simultaneous_downloads);
    let threadpool = Threadpool::new(10);
    let (tx_to_main, rx_to_main) = mpsc::channel();

    for pod in podcast_list.iter() {
        check_feed(
            pod.clone(),
            // config.max_retries,
            3,
            &threadpool,
            tx_to_main.clone(),
        );
    }

    let mut msg_counter: usize = 0;
    let mut failure = false;
    while let Some(message) = rx_to_main.iter().next() {
        match message {
            Msg::Podcast(PCMsg::NewData(pod)) => {
                let title = pod.title.clone();
                let db_result = db_inst.insert_podcast(pod);
                match db_result {
                    Ok(_) => {
                        println!("Added {title}");
                    }
                    Err(_err) => {
                        failure = true;
                        eprintln!("Error adding {title}");
                    }
                }
            }

            Msg::Podcast(PCMsg::Error(feed)) => {
                failure = true;
                if let Some(t) = feed.title {
                    eprintln!("Error retrieving RSS feed: {t}");
                } else {
                    eprintln!("Error retrieving RSS feed");
                }
            }
            _ => (),
        }

        msg_counter += 1;
        if msg_counter >= podcast_list.len() {
            break;
        }
    }

    if failure {
        return Err(anyhow!("Process finished with errors."));
    } else {
        println!("Import successful.");
    }
    return Ok(());
}

/// Exports all podcasts to OPML format, either printing to stdout or
/// exporting to a file.
pub fn export(db_path: &Path, file: &str) -> Result<()> {
    let db_inst = Database::connect(db_path)?;
    let podcast_list = db_inst.get_podcasts()?;
    let opml = export_opml(podcast_list);

    let xml = opml
        .to_string()
        .map_err(|err| anyhow!(err))
        .with_context(|| "Could not create OPML format")?;

    let mut dst =
        File::create(file).with_context(|| format!("Could not create output file: {file}"))?;
    dst.write_all(xml.as_bytes())
        .with_context(|| format!("Could not copy OPML data to output file: {file}"))?;
    return Ok(());
}

/// Import a list of podcast feeds from an OPML file. Supports
/// v1.0, v1.1, and v2.0 OPML files.
fn import_opml(xml: String) -> Result<Vec<PodcastFeed>> {
    return match OPML::from_str(&xml) {
        Err(err) => Err(anyhow!(err)),
        Ok(opml) => {
            let mut feeds = Vec::new();
            for pod in opml.body.outlines.into_iter() {
                if pod.xml_url.is_some() {
                    // match against title attribute first -- if this is
                    // not set or empty, then match against the text
                    // attribute; this must be set, but can be empty
                    let temp_title = pod.title.filter(|t| !t.is_empty());
                    let title = match temp_title {
                        Some(t) => Some(t),
                        None => {
                            if pod.text.is_empty() {
                                None
                            } else {
                                Some(pod.text)
                            }
                        }
                    };
                    feeds.push(PodcastFeed::new(None, pod.xml_url.unwrap(), title));
                }
            }
            Ok(feeds)
        }
    };
}

/// Converts the current set of podcast feeds to the OPML format
fn export_opml(podcasts: Vec<Podcast>) -> OPML {
    let date = Utc::now();
    let mut opml = OPML {
        head: Some(Head {
            title: Some("Shellcaster Podcast Feeds".to_string()),
            date_created: Some(date.to_rfc2822()),
            ..Head::default()
        }),
        ..Default::default()
    };

    let mut outlines = Vec::new();

    for pod in podcasts.iter() {
        // opml.add_feed(&pod.title, &pod.url);
        outlines.push(Outline {
            text: pod.title.clone(),
            r#type: Some("rss".to_string()),
            xml_url: Some(pod.url.clone()),
            title: Some(pod.title.clone()),
            ..Outline::default()
        });
    }

    opml.body = Body { outlines: outlines };
    return opml;
}