use std::path::{Path, PathBuf};
use std::time::Duration;

use ahash::AHashMap;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use episode_db::EpisodeDB;
use file_db::FileDB;
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::{params, Connection};
use semver::Version;

use super::{Episode, EpisodeNoId, NewEpisode, Podcast, PodcastNoId};
use crate::track::Track;
use podcast_db::{PodcastDB, PodcastDBInsertable};

mod episode_db;
mod file_db;
mod migration;
mod podcast_db;

lazy_static! {
    /// Regex for removing "A", "An", and "The" from the beginning of
    /// podcast titles
    static ref RE_ARTICLES: Regex = Regex::new(r"^(a|an|the) ").expect("Regex error.");
}

/// The id type used in the podcast database
pub type PodcastDBId = i64;

#[derive(Debug)]
pub struct SyncResult {
    pub added: Vec<NewEpisode>,
    pub updated: Vec<i64>,
}

/// Struct holding a sqlite database connection, with methods to interact
/// with this connection.
#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    conn: Connection,
}

impl Database {
    /// Creates a new connection to the database (and creates database if
    /// it does not already exist).
    ///
    /// # Errors
    ///
    /// - if creating / opening the database fails
    /// - if migration fails
    pub fn new(path: &Path) -> Result<Database> {
        let mut db_path = path.to_path_buf();
        std::fs::create_dir_all(&db_path).context("Unable to create subdirectory for database.")?;
        db_path.push("data.db");
        let conn = Connection::open(&db_path)?;

        migration::migrate(&conn).context("Database creation / migration")?;

        // SQLite defaults to foreign key support off
        conn.execute("PRAGMA foreign_keys=ON;", [])
            .context("Could not set database parameters.")?;

        Ok(Database {
            path: db_path,
            conn,
        })
    }

    /// Inserts a new podcast and list of podcast episodes into the
    /// database.
    pub fn insert_podcast(&self, podcast: &PodcastNoId) -> Result<SyncResult> {
        let mut conn = Connection::open(&self.path).context("Error connecting to database.")?;
        let tx = conn.transaction()?;

        PodcastDBInsertable::from(podcast).insert_podcast(&tx)?;

        let pod_id: PodcastDBId = {
            let mut stmt = tx.prepare_cached("SELECT id FROM podcasts WHERE url = ?")?;
            stmt.query_row(params![podcast.url], |row| row.get(0))?
        };
        let mut ep_ids = Vec::new();
        for ep in podcast.episodes.iter().rev() {
            let id = Self::insert_episode(&tx, pod_id, ep)?;
            let new_ep = NewEpisode {
                id,
                pod_id,
                title: ep.title.clone(),
                pod_title: podcast.title.clone(),
                selected: false,
            };
            ep_ids.push(new_ep);
        }
        tx.commit()?;

        Ok(SyncResult {
            added: ep_ids,
            updated: Vec::new(),
        })
    }

    /// Inserts a podcast episode into the database.
    pub fn insert_episode(
        conn: &Connection,
        podcast_id: PodcastDBId,
        episode: &EpisodeNoId,
    ) -> Result<PodcastDBId> {
        let pubdate = episode.pubdate.map(|dt| dt.timestamp());

        let mut stmt = conn.prepare_cached(
            "INSERT INTO episodes (podcast_id, title, url, guid,
                description, pubdate, duration, played, hidden, last_position, image_url)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
        )?;
        stmt.execute(params![
            podcast_id,
            episode.title,
            episode.url,
            episode.guid,
            episode.description,
            pubdate,
            episode.duration,
            false,
            false,
            0,
            episode.image_url,
        ])?;
        Ok(conn.last_insert_rowid())
    }

    /// Inserts a filepath to a downloaded episode.
    pub fn insert_file(&self, episode_id: PodcastDBId, path: &Path) -> Result<()> {
        let mut stmt = self.conn.prepare_cached(
            "INSERT INTO files (episode_id, path)
                VALUES (?, ?);",
        )?;
        stmt.execute(params![episode_id, path.to_str()])?;
        Ok(())
    }

    /// Removes a file listing for an episode from the database when the
    /// user has chosen to delete the file.
    pub fn remove_file(&self, episode_id: PodcastDBId) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare_cached("DELETE FROM files WHERE episode_id = ?;")?;
        stmt.execute(params![episode_id])?;
        Ok(())
    }

    /// Removes all file listings for the selected episode ids.
    pub fn remove_files(&self, episode_ids: &[PodcastDBId]) -> Result<()> {
        // convert list of episode ids into a comma-separated String
        let episode_list: Vec<String> = episode_ids
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        let episodes = episode_list.join(", ");

        let mut stmt = self
            .conn
            .prepare_cached("DELETE FROM files WHERE episode_id = (?);")?;
        stmt.execute(params![episodes])?;
        Ok(())
    }

    /// Removes a podcast, all episodes, and files from the database.
    pub fn remove_podcast(&self, podcast_id: PodcastDBId) -> Result<()> {
        // Note: Because of the foreign key constraints on `episodes`
        // and `files` tables, all associated episodes for this podcast
        // will also be deleted, and all associated file entries for
        // those episodes as well.
        let mut stmt = self
            .conn
            .prepare_cached("DELETE FROM podcasts WHERE id = ?;")?;
        stmt.execute(params![podcast_id])?;
        Ok(())
    }

    /// Updates an existing podcast in the database, where metadata is
    /// changed if necessary, and episodes are updated (modified episodes
    /// are updated, new episodes are inserted).
    pub fn update_podcast(&self, pod_id: PodcastDBId, podcast: &PodcastNoId) -> Result<SyncResult> {
        PodcastDBInsertable::from(podcast).update_podcast(pod_id, &self.conn)?;

        let result = self.update_episodes(pod_id, &podcast.title, &podcast.episodes)?;
        Ok(result)
    }

    /// Updates metadata about episodes that already exist in database,
    /// or inserts new episodes.
    ///
    /// Episodes are checked against the URL and published data in
    /// order to determine if they already exist. As such, an existing
    /// episode that has changed either of these fields will show up as
    /// a "new" episode. The old version will still remain in the
    /// database.
    fn update_episodes(
        &self,
        podcast_id: PodcastDBId,
        podcast_title: &str,
        episodes: &[EpisodeNoId],
    ) -> Result<SyncResult> {
        let old_episodes = self.get_episodes(podcast_id, true)?;
        let mut old_ep_map = AHashMap::new();
        for ep in &old_episodes {
            if !ep.guid.is_empty() {
                old_ep_map.insert(&ep.guid, ep);
            }
        }

        let mut conn = Connection::open(&self.path).context("Error connecting to database.")?;
        let tx = conn.transaction()?;

        let mut insert_ep = Vec::new();
        let mut update_ep = Vec::new();
        for new_ep in episodes.iter().rev() {
            let new_pd = new_ep.pubdate.map(|dt| dt.timestamp());

            let mut existing_id = None;
            let mut update = false;

            // primary matching mechanism: check guid to see if it
            // already exists in database
            if !new_ep.guid.is_empty() {
                if let Some(old_ep) = old_ep_map.get(&new_ep.guid) {
                    existing_id = Some(old_ep.id);
                    update = Self::check_for_updates(old_ep, new_ep);
                }
            }

            // fallback matching: for each existing episode, check the
            // title, url, and pubdate -- if two of the three match, we
            // count it as an existing episode; otherwise, we add it as
            // a new episode
            if existing_id.is_none() {
                for old_ep in old_episodes.iter().rev() {
                    let mut matching = 0;
                    matching += i32::from(new_ep.title == old_ep.title);
                    matching += i32::from(new_ep.url == old_ep.url);

                    if let Some(pd) = new_pd {
                        if let Some(old_pd) = old_ep.pubdate {
                            matching += i32::from(pd == old_pd.timestamp());
                        }
                    }

                    if matching >= 2 {
                        existing_id = Some(old_ep.id);
                        update = Self::check_for_updates(old_ep, new_ep);
                        break;
                    }
                }
            }

            if let Some(id) = existing_id {
                if update {
                    let mut stmt = tx.prepare_cached(
                        "UPDATE episodes SET title = ?, url = ?,
                                guid = ?, description = ?, pubdate = ?,
                                duration = ? WHERE id = ?;",
                    )?;
                    stmt.execute(params![
                        new_ep.title,
                        new_ep.url,
                        new_ep.guid,
                        new_ep.description,
                        new_pd,
                        new_ep.duration,
                        id,
                    ])?;
                    update_ep.push(id);
                }
            } else {
                let id = Self::insert_episode(&tx, podcast_id, new_ep)?;
                let new_ep = NewEpisode {
                    id,
                    pod_id: podcast_id,
                    title: new_ep.title.clone(),
                    pod_title: podcast_title.to_string(),
                    selected: false,
                };
                insert_ep.push(new_ep);
            }
        }
        tx.commit()?;
        Ok(SyncResult {
            added: insert_ep,
            updated: update_ep,
        })
    }

    /// Checks two matching episodes to see whether there are details
    /// that need to be updated (e.g., same episode, but the title has
    /// been changed).
    fn check_for_updates(old_ep: &Episode, new_ep: &EpisodeNoId) -> bool {
        let new_pd = new_ep.pubdate.map(|dt| dt.timestamp());
        let mut pd_match = false;
        if let Some(pd) = new_pd {
            if let Some(old_pd) = old_ep.pubdate {
                pd_match = pd == old_pd.timestamp();
            }
        }
        if !(new_ep.title == old_ep.title
            && new_ep.url == old_ep.url
            && new_ep.guid == old_ep.guid
            && new_ep.description == old_ep.description
            && new_ep.duration == old_ep.duration
            && pd_match)
        {
            return true;
        }
        false
    }

    /// Updates an episode to mark it as played or unplayed.
    pub fn set_played_status(&self, episode_id: PodcastDBId, played: bool) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare_cached("UPDATE episodes SET played = ? WHERE id = ?;")?;
        stmt.execute(params![played, episode_id])?;
        Ok(())
    }

    /// Updates an episode to mark it as played or unplayed.
    pub fn set_all_played_status(
        &self,
        episode_id_vec: &[PodcastDBId],
        played: bool,
    ) -> Result<()> {
        let mut conn = Connection::open(&self.path).context("Error connecting to database.")?;
        let tx = conn.transaction()?;

        for episode_id in episode_id_vec {
            let mut stmt = tx.prepare_cached("UPDATE episodes SET played = ? WHERE id = ?;")?;
            stmt.execute(params![played, episode_id])?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Updates an episode to "remove" it by hiding it. "Removed"
    /// episodes need to stay in the database so that they don't get
    /// re-added when the podcast is synced again.
    pub fn hide_episode(&self, episode_id: PodcastDBId, hide: bool) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare_cached("UPDATE episodes SET hidden = ? WHERE id = ?;")?;
        stmt.execute(params![hide, episode_id])?;
        Ok(())
    }

    /// Generates list of all podcasts in database.
    /// TODO: This should probably use a JOIN statement instead.
    pub fn get_podcasts(&self) -> Result<Vec<Podcast>> {
        let mut stmt = self.conn.prepare_cached("SELECT * FROM podcasts;")?;
        let podcasts = stmt
            .query_map([], PodcastDB::try_from_row_named)?
            .flatten()
            .map(|podcast| {
                let episodes = match self.get_episodes(podcast.id, false) {
                    Ok(ep_list) => Ok(ep_list),
                    Err(_) => Err(rusqlite::Error::QueryReturnedNoRows),
                }?;

                let title_lower = podcast.title.to_lowercase();
                let sort_title = RE_ARTICLES.replace(&title_lower, "").to_string();

                Ok(Podcast {
                    id: podcast.id,
                    title: podcast.title,
                    sort_title,
                    url: podcast.url,
                    description: podcast.description,
                    author: podcast.author,
                    explicit: podcast.explicit,
                    last_checked: podcast.last_checked,
                    episodes,
                    image_url: podcast.image_url,
                })
            })
            .collect::<Result<_, rusqlite::Error>>()?;

        Ok(podcasts)
    }

    /// Generates list of episodes for a given podcast.
    pub fn get_episodes(&self, pod_id: PodcastDBId, include_hidden: bool) -> Result<Vec<Episode>> {
        let mut stmt = if include_hidden {
            self.conn.prepare_cached(
                "SELECT episodes.id as epid, files.id as fileid, * FROM episodes
                        LEFT JOIN files ON episodes.id = files.episode_id
                        WHERE episodes.podcast_id = ?
                        ORDER BY pubdate DESC;",
            )?
        } else {
            self.conn.prepare_cached(
                "SELECT episodes.id as epid, files.id as fileid, * FROM episodes
                        LEFT JOIN files ON episodes.id = files.episode_id
                        WHERE episodes.podcast_id = ?
                        AND episodes.hidden = 0
                        ORDER BY pubdate DESC;",
            )?
        };

        let episodes = stmt
            .query_map(params![pod_id], |row| {
                let episode = EpisodeDB::try_from_row_named_alias_id(row)?;
                let file = FileDB::try_from_row_named_alias_id(row).ok();

                Ok(Episode {
                    id: episode.id,
                    pod_id,
                    title: episode.title,
                    url: episode.url,
                    guid: episode.guid,
                    description: episode.description,
                    pubdate: episode.pubdate,
                    duration: episode.duration,
                    path: file.map(|v| v.path),
                    played: episode.played,
                    last_position: episode.last_position,
                    image_url: episode.image_url,
                })
            })?
            .flatten()
            .collect();

        Ok(episodes)
    }

    /// Deletes all rows in all tables
    pub fn clear_db(&self) -> Result<()> {
        self.conn.execute("DELETE FROM files;", [])?;
        self.conn.execute("DELETE FROM episodes;", [])?;
        self.conn.execute("DELETE FROM podcasts;", [])?;
        Ok(())
    }

    pub fn get_last_position(&mut self, track: &Track) -> Result<Duration> {
        let query = "SELECT last_position FROM episodes WHERE url = ?1";

        let mut last_position: Duration = Duration::from_secs(0);
        self.conn.query_row(
            query,
            params![track.file().unwrap_or("Unknown File").to_string(),],
            |row| {
                let last_position_u64: u64 = row.get(0)?;
                // error!("last_position_u64 is {last_position_u64}");
                last_position = Duration::from_secs(last_position_u64);
                Ok(last_position)
            },
        )?;
        // error!("get last pos as {}", last_position.as_secs());
        Ok(last_position)
    }

    /// # Errors
    ///
    /// - if the connection is unavailable
    /// - if the query fails
    pub fn set_last_position(&self, track: &Track, last_position: Duration) -> Result<()> {
        let query = "UPDATE episodes SET last_position = ?1 WHERE url = ?2";
        self.conn
            .execute(
                query,
                params![
                    last_position.as_secs(),
                    track.file().unwrap_or("Unknown File Name").to_string(),
                ],
            )
            .context("update last position failed.")?;
        // error!("set last position as {}", last_position.as_secs());

        Ok(())
    }
}

/// Helper function converting an (optional) Unix timestamp to a
/// `DateTime`<Utc> object
fn convert_date(result: &Result<i64, rusqlite::Error>) -> Option<DateTime<Utc>> {
    match result {
        Ok(timestamp) => DateTime::from_timestamp(*timestamp, 0),
        Err(_) => None,
    }
}

#[cfg(test)]
mod test_utils {
    use rusqlite::Connection;

    /// Open a new In-Memory sqlite database
    pub fn gen_database() -> Connection {
        Connection::open_in_memory().expect("open db failed")
    }
}
