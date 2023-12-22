use reqwest::Url;
use serde::{de, Deserialize, Serialize};

// OMDB only has movies
#[derive(Debug)]
pub struct OMDB {
    key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OMDBResult {
    #[serde(rename = "Response")]
    #[serde(deserialize_with = "deserialize_response")]
    pub response: bool,
    #[serde(rename = "Title")]
    pub title: Option<String>,
    #[serde(rename = "Year")]
    pub year: Option<String>,
    #[serde(rename = "Error")]
    pub error: Option<String>,
}

fn deserialize_response<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;

    match s {
        "True" => Ok(true),
        "False" => Ok(false),
        _ => Err(de::Error::unknown_variant(s, &["True", "False"])),
    }
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

        match reqwest::blocking::get(url.as_str()) {
            Ok(r) => match r.json::<OMDBResult>() {
                Ok(j) => {
                    if j.response {
                        Some(j)
                    } else {
                        println!("Error: {}", j.error.unwrap());
                        None
                    }
                }
                Err(e) => {
                    println!("Cannot read json response: {e}");
                    None
                }
            },
            Err(e) => {
                println!("Cannot get movie info: {e}");
                None
            }
        }
    }
}
