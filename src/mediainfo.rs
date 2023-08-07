use crate::mediainfo::MediaInfo::{Movie, NoMedia, TVShow};
use crate::omdb::OMDB;
use crate::tvmaze::TVMaze;
use std::path::PathBuf;

#[derive(Debug)]
pub struct TVShowInfo {
    pub title: String,
    pub season: u8,
    pub episode: u8,
}

#[derive(Debug)]
pub struct MovieInfo {
    pub title: String,
    pub year: String,
}

#[derive(Debug)]
pub enum MediaInfo {
    Movie { info: MovieInfo },
    TVShow { info: TVShowInfo },
    NoMedia { path: PathBuf },
}

impl MediaInfo {
    pub fn from_path(path: &PathBuf, omdb_apikey: &str) -> anyhow::Result<MediaInfo> {
        match path.extension() {
            None => return Ok(NoMedia { path: path.clone() }),
            Some(ext) => match ext.to_str() {
                Some("mkv") | Some("avi") | Some("mp4") => println!("Media file"),
                _ => return Ok(NoMedia { path: path.clone() }),
            },
        }

        let title = Self::path_normalize(&path);

        Ok(match Self::extract_show_season_episode(&title) {
            Some(i) => match TVMaze::search_show(&i.title) {
                Some(res) => TVShow {
                    info: TVShowInfo {
                        title: res.show.name,
                        season: i.season,
                        episode: i.episode,
                    },
                },
                None => NoMedia { path: path.clone() }, // TODO: This should be an error
            },
            None => {
                let omdb = OMDB::new(omdb_apikey);
                match omdb.search_movie(&title) {
                    Some(res) => Movie {
                        info: MovieInfo {
                            title: res.title,
                            year: res.year,
                        },
                    },
                    None => NoMedia { path: path.clone() }, // TODO: This should be an error
                }
            }
        })
    }

    fn path_normalize(path: &PathBuf) -> String {
        let punctuation = regex::Regex::new(r"[\.\-_]").unwrap();
        let encodings = regex::Regex::new(
            r"(720p|1080p|1440p|2160p|hdtv|x264|dts|bluray|aac|atmos|x265|hevc|h264|h265|web|webrip).*",
        )
            .unwrap();
        let mut path = path.clone();

        path.set_extension("");

        let mut name = if let Some(file_name) = path.file_name() {
            if let Some(lower) = file_name.to_ascii_lowercase().to_str() {
                lower.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        name = punctuation.replace_all(&name, " ").to_string();
        name = encodings.replace(&name, "").to_string();

        name.trim().to_string()
    }

    fn extract_show_season_episode(name: &str) -> Option<TVShowInfo> {
        let se =
            regex::Regex::new(r"(?P<title>.*)[Ss](?P<season>\d{1,2})[Ee](?P<episode>\d{1,2}).*")
                .unwrap();
        let caps = match se.captures(name) {
            None => return None,
            Some(c) => c,
        };
        let season: u8 = if let Ok(s) = caps["season"].parse() {
            s
        } else {
            return None;
        };

        let episode: u8 = if let Ok(e) = caps["episode"].parse() {
            e
        } else {
            return None;
        };

        Some(TVShowInfo {
            title: caps["title"].to_string(),
            season,
            episode,
        })
    }
}
