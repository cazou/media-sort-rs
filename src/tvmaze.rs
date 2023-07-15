use reqwest::Url;
use serde::{Deserialize, Serialize};

// TVMaze only has shows
#[derive(Debug)]
pub struct TVMaze;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShowResult {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub score: f64,
    pub show: ShowResult,
}

impl TVMaze {
    pub fn search_show(title: &str) -> Option<SearchResult> {
        let url =
            Url::parse_with_params("http://api.tvmaze.com/search/shows", &[("q", title.trim())])
                .unwrap();

        let resp = match reqwest::blocking::get(url.as_str()) {
            Ok(r) => match r.json::<Vec<SearchResult>>() {
                Ok(j) => j,
                Err(e) => {
                    println!("Cannot read json response: {e}");
                    return None;
                }
            },
            Err(e) => {
                println!("Cannot get show info: {e}");
                return None;
            }
        };

        return if resp.is_empty() {
            None
        } else {
            Some(resp[0].clone())
        };
    }
}
