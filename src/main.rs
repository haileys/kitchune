extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::env;

mod ui;

fn main() {
    let spotify_creds = env::var("KITCHUNE_SPOTIFY_CREDS")
        .expect("KITCHUNE_SPOTIFY_CREDS to be set");

    let sdl = sdl2::init().expect("sdl2::init");

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

    let model = ui::Model {
        track_name: "Because Of You",
        track_artist: "S.P.Y, Etherwood",
        playing: true,
        saved: false,
    };

    match ui::render_to_canvas(&mut canvas, &fonts, &model) {
        Ok(()) => {}
        Err(e) => { eprintln!("render_to_canvas: {:?}", e) }
    }

    let mut event_pump = sdl.event_pump().unwrap();

    for event in event_pump.wait_iter() {
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                break;
            }
            _ => {}
        }
    }
}
