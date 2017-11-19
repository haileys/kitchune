use std::convert::From;
use std::default::Default;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas, TextureValueError};
use sdl2::video::WindowContext;
use sdl2::ttf::{Font, FontError};

pub const WIDTH: u32 = 480;
pub const HEIGHT: u32 = 320;

const ICON_PLAY: &'static str = "\u{e037}";
const ICON_PAUSE: &'static str = "\u{e034}";
const ICON_SKIP_NEXT: &'static str = "\u{e044}";
const ICON_TICK: &'static str = "\u{e876}";
const ICON_ADD: &'static str = "\u{e145}";

pub struct Fonts<'a> {
    pub opensans_24: Font<'a, 'static>,
    pub opensans_18: Font<'a, 'static>,
    pub material_128: Font<'a, 'static>,
}

pub struct Model {
    pub track_name: String,
    pub track_artist: String,
    pub playing: bool,
    pub saved: bool,
}

impl Default for Model {
    fn default() -> Self {
        Model {
            track_name: String::new(),
            track_artist: String::new(),
            playing: false,
            saved: false,
        }
    }
}

#[derive(Debug)]
pub enum RenderError {
    String(String),
    Font(FontError),
    Texture(TextureValueError),
}

impl From<String> for RenderError {
    fn from(val: String) -> RenderError {
        RenderError::String(val)
    }
}

impl From<FontError> for RenderError {
    fn from(val: FontError) -> RenderError {
        RenderError::Font(val)
    }
}

impl From<TextureValueError> for RenderError {
    fn from(val: TextureValueError) -> RenderError {
        RenderError::Texture(val)
    }
}

fn render_text<'a>(texture_creator: &'a TextureCreator<WindowContext>, font: &Font, text: &str) -> Result<Option<Texture<'a>>, RenderError> {
    let surface = font.render(&text)
        .blended(Color::RGB(255, 255, 255));

    match surface {
        Err(FontError::SdlError(ref s)) if s == "Text has zero width" => {
            Ok(None)
        }
        Err(e) => {
            Err(e.into())
        }
        Ok(surface) => {
            texture_creator.create_texture_from_surface(surface).map(Some).map_err(|e| e.into())
        }
    }
}

pub fn render_to_canvas(canvas: &mut WindowCanvas, fonts: &Fonts, model: &Model)
    -> Result<(), RenderError>
{
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGB(40, 40, 40));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(160, 160, 160));
    canvas.fill_rect(Rect::new(16, 16, 96, 96))?;

    // render track name:
    if let Some(track_name_tex) = render_text(&texture_creator, &fonts.opensans_24, &model.track_name)? {
        let track_name_dim = track_name_tex.query();

        canvas.copy(&track_name_tex, None, Some(
            Rect::new(140, 32, track_name_dim.width, track_name_dim.height)))?;
    }

    // render track artist:
    if let Some(track_artist_tex) = render_text(&texture_creator, &fonts.opensans_18, &model.track_artist)? {
        let track_artist_dim = track_artist_tex.query();

        canvas.copy(&track_artist_tex, None, Some(
            Rect::new(140, 64, track_artist_dim.width, track_artist_dim.height)))?;
    }

    // render buttons:

    let button_y = 144;
    let play_pause_x;

    {
        let play_pause_tex = texture_creator.create_texture_from_surface(
            fonts.material_128
                .render(if model.playing {
                    ICON_PAUSE
                } else {
                    ICON_PLAY
                })
                .blended(Color::RGB(169, 169, 169))?)?;

        let play_pause_dim = play_pause_tex.query();

        play_pause_x = ((WIDTH - play_pause_dim.width) / 2) as i32;

        canvas.copy(&play_pause_tex, None, Some(
            Rect::new(
                play_pause_x,
                button_y,
                play_pause_dim.width,
                play_pause_dim.height)))?;
    }

    {
        let save_tex = texture_creator.create_texture_from_surface(
            fonts.material_128
                .render(if model.saved {
                    ICON_TICK
                } else {
                    ICON_ADD
                })
                .blended(Color::RGB(169, 169, 169))?)?;

        let save_dim = save_tex.query();

        canvas.copy(&save_tex, None, Some(
            Rect::new(
                play_pause_x - 128,
                button_y,
                save_dim.width,
                save_dim.height)))?;
    }

    {
        let skip_tex = texture_creator.create_texture_from_surface(
            fonts.material_128
                .render(ICON_SKIP_NEXT)
                .blended(Color::RGB(169, 169, 169))?)?;

        let skip_dim = skip_tex.query();

        canvas.copy(&skip_tex, None, Some(
            Rect::new(
                play_pause_x + 128,
                button_y,
                skip_dim.width,
                skip_dim.height)))?;
    }

    canvas.present();

    Ok(())
}
