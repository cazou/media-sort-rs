use std::fmt::{Display, Formatter};
//use crate::mediainfo::MediaInfo::{Movie, NoMedia, TVShow};
use crate::omdb::OMDB;
use crate::tvmaze::TVMaze;
use anyhow::bail;
use chrono::{Datelike, Utc};
use std::path::PathBuf;

#[derive(Eq, PartialEq, Debug)]
pub enum Episode {
    Numbered(u8),
    Special(String),
}

impl Display for Episode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Episode::Numbered(e) => write!(f, "{:02}", e),
            Episode::Special(t) => write!(f, "00 - {}", t),
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct TVShowInfo {
    pub season: u8,
    pub episode: Episode,
}

#[derive(Eq, PartialEq, Debug)]
pub struct MediaInfo {
    pub name: String,
    pub year: Option<i32>,
    pub show_info: Option<TVShowInfo>,
}

impl MediaInfo {
    pub fn is_show(&self) -> bool {
        self.show_info.is_some()
    }

    pub fn from_path(path: &PathBuf, omdb_apikey: &str) -> anyhow::Result<MediaInfo> {
        match path.extension() {
            None => bail!("No extension: {}", path.to_str().unwrap_or("")),
            Some(ext) => match ext.to_str() {
                Some("mkv") | Some("avi") | Some("mp4") | Some("srt") => {}
                _ => bail!("Unknown extension: {}", path.to_str().unwrap_or("")),
            },
        }

        let media_info = Self::extract_media_info(&path);

        Ok(match media_info.show_info {
            Some(i) => match TVMaze::search_show(&media_info.name, media_info.year) {
                Some(res) => MediaInfo {
                    name: res.show.name,
                    year: media_info.year,
                    show_info: Some(TVShowInfo {
                        season: i.season,
                        episode: i.episode,
                    }),
                },
                None => bail!(
                    "Show not found: {} ({})",
                    &media_info.name,
                    media_info.year.unwrap_or(-1)
                ),
            },
            None => {
                let omdb = OMDB::new(omdb_apikey);
                match omdb.search_movie(&media_info.name, media_info.year) {
                    Some(res) => MediaInfo {
                        name: res.title,
                        year: media_info.year,
                        show_info: None,
                    },
                    None => bail!(
                        "Movie not found: {} ({})",
                        &media_info.name,
                        media_info.year.unwrap_or(-1)
                    ),
                }
            }
        })
    }

    fn extract_media_info(path: &PathBuf) -> MediaInfo {
        let mut media_info = MediaInfo {
            name: Self::path_normalize(path),
            year: None,
            show_info: None,
        };

        media_info.extract_show_season_episode();

        media_info.extract_year();

        media_info
    }

    fn path_normalize(path: &PathBuf) -> String {
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

        name.trim().to_string()
    }

    fn extract_year(&mut self) {
        /*
         * Estimate if the title is followed by a year. It will word with titles like "The 4400"
         * (4400 is not a valid movie year), but still have an issue with titles like "2012".
         * I've seen that so let's just work with this for now.
         */
        let year_re = regex::Regex::new(r"^(?P<title>.*) (?P<year>\d{4})$").unwrap();

        match year_re.captures(&self.name) {
            None => {}
            Some(c) => match c["year"].parse::<i32>() {
                Ok(y) => {
                    let now = Utc::now();
                    // We consider that the first movie made was "The Horse in Motion" in 1878
                    if (1878..=now.year()).contains(&y) {
                        self.name = c["title"].to_string();
                        self.year = Some(y);
                    }
                }
                Err(_) => {}
            },
        };
    }

    fn extract_show_season_episode(&mut self) {
        let se = regex::Regex::new(
            r"(?P<name>.*)[Ss](?P<season>\d{1,2})[Ee](?P<episode>\d{1,2}) *(?P<title>.*)",
        )
        .unwrap();
        let caps = match se.captures(&self.name) {
            None => return,
            Some(c) => c,
        };
        let season: u8 = if let Ok(s) = caps["season"].parse() {
            s
        } else {
            return;
        };

        let episode: u8 = if let Ok(e) = caps["episode"].parse() {
            e
        } else {
            return;
        };

        let episode = if episode != 0 {
            Episode::Numbered(episode)
        } else if caps["title"].is_empty() {
            Episode::Special("Unknown Special".to_string())
        } else {
            Episode::Special(Self::capitalize_words(&caps["title"]))
        };

        self.name = caps["name"].trim().to_string();
        self.show_info = Some(TVShowInfo { season, episode });
    }

    fn capitalize_words(value: &str) -> String {
        value
            .split(' ')
            .map(|w| {
                let new_word: String = w
                    .char_indices()
                    .map(|(i, c)| {
                        if i == 0 {
                            c.to_uppercase().to_string()
                        } else {
                            c.to_string()
                        }
                    })
                    .collect();
                new_word
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[cfg(test)]
mod mediainfo_tests {
    use crate::mediainfo::Episode::Special;
    use crate::mediainfo::{Episode, MediaInfo, TVShowInfo};
    use std::path::PathBuf;

    #[test]
    fn check_normalize() {
        let path =
            PathBuf::from("Test title 22 (123(4) ) ) h264 - (ddd(d)) || )(*&^%$#@ rubbish.mkv");
        assert_eq!(
            MediaInfo::extract_media_info(&path),
            MediaInfo {
                name: String::from("test title 22"),
                year: None,
                show_info: None,
            }
        );

        let path =
            PathBuf::from("Test title 1922 (123(4) ) ) h264 - (ddd(d)) || )(*&^%$#@ rubbish.mkv");
        assert_eq!(
            MediaInfo::extract_media_info(&path),
            MediaInfo {
                name: String::from("test title"),
                year: Some(1922),
                show_info: None,
            }
        );

        let path = PathBuf::from(
            "Test 2022 title 42 (123(4) ) ) h264 - (ddd(d)) || )(*&^%$#@ rubbish.mkv",
        );
        assert_eq!(
            MediaInfo::extract_media_info(&path),
            MediaInfo {
                name: String::from("test 2022 title 42"),
                year: None,
                show_info: None,
            }
        );

        let path = PathBuf::from(
            "Great.Series.2005.s13e00.special.title.1080p.web.h264-ggez[eztv.re].mkv",
        );
        assert_eq!(
            MediaInfo::extract_media_info(&path),
            MediaInfo {
                name: String::from("great series"),
                year: Some(2005),
                show_info: Some(TVShowInfo {
                    season: 13,
                    episode: Special(String::from("Special Title")),
                }),
            }
        );

        let path = PathBuf::from("Great.Series.2005.s13e00.1080p.web.h264-ggez[eztv.re].mkv");
        assert_eq!(
            MediaInfo::extract_media_info(&path),
            MediaInfo {
                name: String::from("great series"),
                year: Some(2005),
                show_info: Some(TVShowInfo {
                    season: 13,
                    episode: Special(String::from("Unknown Special")),
                }),
            }
        );

        let path = PathBuf::from(
            "Great.Series.2005.s13e03.episode.title.1080p.web.h264-ggez[eztv.re].mkv",
        );
        assert_eq!(
            MediaInfo::extract_media_info(&path),
            MediaInfo {
                name: String::from("great series"),
                year: Some(2005),
                show_info: Some(TVShowInfo {
                    season: 13,
                    episode: Episode::Numbered(3),
                }),
            }
        );
    }
}
