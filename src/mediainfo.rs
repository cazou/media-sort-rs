use crate::mediainfo::MediaInfo::{Movie, NoMedia, TVShow};
use crate::omdb::OMDB;
use crate::tvmaze::TVMaze;
use chrono::{Datelike, Utc};
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

#[derive(Eq, PartialEq, Debug)]
struct SearchInfo {
    pub title: String,
    pub year: Option<i32>,
}

impl MediaInfo {
    pub fn from_path(path: &PathBuf, omdb_apikey: &str) -> anyhow::Result<MediaInfo> {
        match path.extension() {
            None => return Ok(NoMedia { path: path.clone() }),
            Some(ext) => match ext.to_str() {
                Some("mkv") | Some("avi") | Some("mp4") | Some("srt") => println!("Media file"),
                _ => return Ok(NoMedia { path: path.clone() }),
            },
        }

        let search_info = Self::path_normalize(&path);

        Ok(
            match Self::extract_show_season_episode(&search_info.title) {
                Some(i) => match TVMaze::search_show(&i.title, search_info.year) {
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
                    match omdb.search_movie(&search_info.title, search_info.year) {
                        Some(res) => Movie {
                            info: MovieInfo {
                                title: res.title,
                                year: res.year,
                            },
                        },
                        None => NoMedia { path: path.clone() }, // TODO: This should be an error
                    }
                }
            },
        )
    }

    fn path_normalize(path: &PathBuf) -> SearchInfo {
        let punctuation = regex::Regex::new(r"[\.\-_]").unwrap();
        let encodings = regex::Regex::new(
            r"(720p|1080p|1440p|2160p|hdtv|x264|dts|bluray|aac|atmos|x265|hevc|h264|h265|web|webrip|imax).*",
        )
            .unwrap();
        let parenthesis = regex::Regex::new(r"\(.*\)").unwrap();
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
        name = parenthesis.replace_all(&name, "").to_string();

        /*
         * Estimate if the title is followed by a year. It will word with titles like "The 4400"
         * (4400 is not a valid movie year), but still have an issue with titles like "2012".
         * I've seen that so let's just work with this for now.
         */

        let name = name.trim().to_string();

        let year_re = regex::Regex::new(r"^(?P<title>.*) (?P<year>\d{4})$").unwrap();

        let (title, year) = match year_re.captures(&name) {
            None => (name, None),
            Some(c) => match c["year"].parse::<i32>() {
                Ok(y) => {
                    let now = Utc::now();
                    // We consider that the first movie made was "The Horse in Motion" in 1878
                    if (1878..=now.year()).contains(&y) {
                        (c["title"].to_string(), Some(y))
                    } else {
                        (name, None)
                    }
                }
                Err(_) => (name, None),
            },
        };

        SearchInfo { title, year }
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

#[cfg(test)]
mod mediainfo_tests {
    use crate::mediainfo::{MediaInfo, SearchInfo};
    use std::path::PathBuf;

    #[test]
    fn check_normalize() {
        let path =
            PathBuf::from("Test title 22 (123(4) ) ) h264 - (ddd(d)) || )(*&^%$#@ rubbish.mkv");
        assert_eq!(
            MediaInfo::path_normalize(&path),
            SearchInfo {
                title: String::from("test title 22"),
                year: None
            }
        );

        let path =
            PathBuf::from("Test title 1922 (123(4) ) ) h264 - (ddd(d)) || )(*&^%$#@ rubbish.mkv");
        assert_eq!(
            MediaInfo::path_normalize(&path),
            SearchInfo {
                title: String::from("test title"),
                year: Some(1922)
            }
        );

        let path = PathBuf::from(
            "Test 2022 title 42 (123(4) ) ) h264 - (ddd(d)) || )(*&^%$#@ rubbish.mkv",
        );
        assert_eq!(
            MediaInfo::path_normalize(&path),
            SearchInfo {
                title: String::from("test 2022 title 42"),
                year: None
            }
        );
    }
}
