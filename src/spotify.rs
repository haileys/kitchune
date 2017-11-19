use reqwest::{Client, Error, Method, RequestBuilder};
use reqwest::header::{Authorization, Bearer};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub artists: Vec<Artist>,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub is_playing: bool,
    #[serde(rename="item")]
    pub track: Option<Track>,
}

pub struct Spotify {
    token: String,
    client: Client,
}

impl Spotify {
    pub fn new(token: String) -> Spotify {
        Spotify {
            token: token,
            client: Client::new(),
        }
    }

    fn build_request(&self, method: Method, path: &str, params: &[(&str, &str)]) -> RequestBuilder {
        let url = format!("https://api.spotify.com/v1{}", path);
        let url = Url::parse_with_params(&url, params).unwrap();
        let mut builder = self.client.request(method, url);
        builder.header(Authorization(Bearer { token: self.token.clone() }));
        builder
    }

    pub fn player(&self) -> Result<Player, Error> {
        self.build_request(Method::Get, "/me/player", &[])
            .send()?
            .json()
    }

    pub fn is_saved_track(&self, track_id: &str) -> Result<bool, Error> {
        self.build_request(Method::Get, "/me/tracks/contains", &[("ids", track_id)])
            .send()?
            .json::<Vec<bool>>()
            .map(|saved| saved.get(0) == Some(&true))
    }
}
