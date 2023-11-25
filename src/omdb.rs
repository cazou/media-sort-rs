use reqwest::Url;
use serde::{Deserialize, Serialize};

// OMDB only has movies
#[derive(Debug)]
pub struct OMDB {
    key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OMDBResult {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Year")]
    pub year: String,
}

impl OMDB {
    pub fn new(key: &str) -> OMDB {
        OMDB {
            key: key.to_string(),
        }
    }

    pub fn search_movie(&self, title: &str, year: Option<i32>) -> Option<OMDBResult> {
        let mut params = vec![("t", title.trim()), ("apikey", self.key.as_str())];
        let year_str;

        if let Some(y) = year {
            year_str = format!("{}", y);
            params.push(("y", year_str.as_str()));
        }

        let url = Url::parse_with_params("http://www.omdbapi.com/", &params).unwrap();

        let resp = match reqwest::blocking::get(url.as_str()) {
            Ok(r) => match r.json::<OMDBResult>() {
                Ok(j) => j,
                Err(e) => {
                    println!("Cannot read json response: {e}");
                    return None;
                }
            },
            Err(e) => {
                println!("Cannot get movie info: {e}");
                return None;
            }
        };

        Some(resp)
    }
}
