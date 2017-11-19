#[macro_use] extern crate serde_derive;

extern crate crossbeam;
extern crate dotenv;
extern crate reqwest;
extern crate sdl2;
extern crate serde;
extern crate serde_json;
extern crate url;

mod spotify;
mod ui;

use std::default::Default;
use std::env;
use std::time::Duration;
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError};

use sdl2::EventSubsystem;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use spotify::Spotify;

struct SpotifyModelUpdate(ui::Model);

fn spotify_poll_thread(kill: Receiver<()>, spotify: &Spotify, event: &EventSubsystem) {
    loop {
        match do_poll(spotify) {
            Ok(model) => {
                event.push_custom_event(SpotifyModelUpdate(model))
                    .expect("push_custom_event");
            }
            Err(e) => {
                println!("spotify_poll_thread: {:?}", e);
            }
        }

        match kill.recv_timeout(Duration::from_millis(1000)) {
            Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => continue,
        }
    }

    fn do_poll(spotify: &Spotify) -> Result<ui::Model, reqwest::Error> {
        let player = spotify.player()?;

        let track = match player.track {
            Some(ref track) => track,
            None => {
                return Ok(ui::Model {
                    track_name: "".to_owned(),
                    track_artist: "".to_owned(),
                    playing: false,
                    saved: false,
                });
            }
        };

        let mut joined_artists = String::new();

        for artist in &track.artists {
            if joined_artists.len() > 0 {
                joined_artists += ", ";
            }

            joined_artists += &artist.name;
        }

        let saved = spotify.is_saved_track(&track.id)?;

        Ok(ui::Model {
            track_name: track.name.clone(),
            track_artist: joined_artists,
            playing: player.is_playing,
            saved: saved,
        })
    }
}

fn main() {
    dotenv::dotenv().ok();

    let spotify = spotify::Spotify::new(
        env::var("KITCHUNE_SPOTIFY_CREDS")
            .expect("KITCHUNE_SPOTIFY_CREDS to be set"));

    let sdl = sdl2::init().expect("sdl2::init");

    let sdl_event = sdl.event().expect("sdl2::event");
    let sdl_event2 = sdl.event().expect("sdl2::event");

    sdl_event.register_custom_event::<SpotifyModelUpdate>()
        .expect("register_custom_event");

    let video = sdl.video().unwrap();

    let window = video.window("Kitchune", ui::WIDTH, ui::HEIGHT)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .build()
        .unwrap();

    let ttf = sdl2::ttf::init().expect("sdl2::ttf::init");

    let fonts = ui::Fonts {
        opensans_24: ttf.load_font("assets/fonts/OpenSans-Regular.ttf", 24).unwrap(),
        opensans_18: ttf.load_font("assets/fonts/OpenSans-Regular.ttf", 18).unwrap(),
        material_128: ttf.load_font("assets/fonts/MaterialIcons-Regular.ttf", 128).unwrap(),
    };

    ui::render_to_canvas(&mut canvas, &fonts, &Default::default())
        .expect("render_to_canvas");

    let (kill_spotify_tx, kill_spotify_rx) = sync_channel(0);

    crossbeam::scope(|scope| {
        scope.spawn(|| {
            spotify_poll_thread(kill_spotify_rx, &spotify, &sdl_event2);
        });

        let mut event_pump = sdl.event_pump().unwrap();

        for event in event_pump.wait_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    kill_spotify_tx.send(()).unwrap();
                    break;
                }
                ref ev if ev.is_user_event() => {
                    if let Some(SpotifyModelUpdate(model)) = ev.as_user_event_type() {
                        ui::render_to_canvas(&mut canvas, &fonts, &model)
                            .expect("render_to_canvas");
                    }
                }
                _ => {}
            }
        }
    })
}
