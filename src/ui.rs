use std::convert::From;
use std::cmp::min;
use std::default::Default;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{WindowCanvas, TextureValueError};
use sdl2::ttf::{Font, FontError};

pub const WIDTH: usize = 480;
pub const HEIGHT: usize = 320;

const BUTTON_SIZE: usize = 128;
const BUTTON_Y: usize = 144;
const BUTTON_SPACING: usize = 16;

const ELLIPSIS: &'static str = "\u{2026}";

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

struct Label<'a> {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: Color,
    font: &'a Font<'a, 'static>,
    text: String,
    overflow: Overflow,
    align: TextAlign,
}

enum Overflow {
    Hidden,
    Ellipsis,
}

enum TextAlign {
    Left,
    Center,
}

pub enum Action {
    Save,
    PlayPause,
    SkipNext,
}

pub struct Ui<'a> {
    title: Label<'a>,
    artist: Label<'a>,
    save: Label<'a>,
    play: Label<'a>,
    next: Label<'a>,
}

impl<'a> Ui<'a> {
    pub fn new(fonts: &'a Fonts) -> Ui<'a> {
        Ui {
            title: Label {
                x: 48,
                y: 48,
                w: WIDTH - 48 * 2,
                h: 32,
                color: Color::RGB(255, 255, 255),
                font: &fonts.opensans_24,
                text: String::new(),
                overflow: Overflow::Ellipsis,
                align: TextAlign::Left,
            },
            artist: Label {
                x: 48,
                y: 80,
                w: WIDTH - 48 * 2,
                h: 32,
                color: Color::RGB(255, 255, 255),
                font: &fonts.opensans_18,
                text: String::new(),
                overflow: Overflow::Ellipsis,
                align: TextAlign::Left,
            },
            save: Label {
                x: (WIDTH / 2) - (BUTTON_SIZE / 2) - BUTTON_SPACING - BUTTON_SIZE,
                y: BUTTON_Y,
                w: BUTTON_SIZE,
                h: BUTTON_SIZE,
                color: Color::RGB(169, 169, 169),
                font: &fonts.material_128,
                text: String::new(),
                overflow: Overflow::Hidden,
                align: TextAlign::Center,
            },
            play: Label {
                x: (WIDTH / 2) - (BUTTON_SIZE / 2),
                y: BUTTON_Y,
                w: BUTTON_SIZE,
                h: BUTTON_SIZE,
                color: Color::RGB(169, 169, 169),
                font: &fonts.material_128,
                text: String::new(),
                overflow: Overflow::Hidden,
                align: TextAlign::Center,
            },
            next: Label {
                x: (WIDTH / 2) + (BUTTON_SIZE / 2) + BUTTON_SPACING,
                y: BUTTON_Y,
                w: BUTTON_SIZE,
                h: BUTTON_SIZE,
                color: Color::RGB(169, 169, 169),
                font: &fonts.material_128,
                text: String::new(),
                overflow: Overflow::Hidden,
                align: TextAlign::Center,
            },
        }
    }

    pub fn update_data(&mut self, model: &Model) {
        self.title.text = model.track_name.to_owned();
        self.artist.text = model.track_artist.to_owned();

        self.save.text = if model.saved {
            ICON_TICK
        } else {
            ICON_ADD
        }.to_owned();

        self.play.text = if model.playing {
            ICON_PAUSE
        } else {
            ICON_PLAY
        }.to_owned();

        self.next.text = ICON_SKIP_NEXT.to_owned();
    }

    fn in_bounds(&self, point: &Point, label: &Label) -> bool {
        let px = point.x as usize;
        let py = point.y as usize;

        px >= label.x && px < (label.x + label.w) &&
        py >= label.y && py < (label.y + label.h)
    }

    pub fn click(&self, point: Point) -> Option<Action> {
        if self.in_bounds(&point, &self.save) {
            Some(Action::Save)
        } else if self.in_bounds(&point, &self.play) {
            Some(Action::PlayPause)
        } else if self.in_bounds(&point, &self.next) {
            Some(Action::SkipNext)
        } else {
            None
        }
    }

    pub fn render(&self, canvas: &mut WindowCanvas) -> Result<(), RenderError> {
        let texture_creator = canvas.texture_creator();

        canvas.set_draw_color(Color::RGB(40, 40, 40));
        canvas.clear();

        for label in &[&self.title, &self.artist, &self.save, &self.play, &self.next] {
            let text = match label.overflow {
                Overflow::Ellipsis => ellipsize_text(label.font, label.text.to_owned(), label.w)?,
                Overflow::Hidden => label.text.to_owned(),
            };

            let surface = label.font.render(&text)
                .blended(label.color.clone());

            let surface = match surface {
                Err(FontError::SdlError(ref s)) if s == "Text has zero width" => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
                Ok(surface) => surface,
            };

            let texture = texture_creator.create_texture_from_surface(surface)?;
            let dimension = texture.query();

            let dim_height = min(label.h, dimension.height as usize);
            let dim_width = min(label.w, dimension.width as usize);

            let (x, y, w, h) = match label.align {
                TextAlign::Left => (
                    label.x,
                    label.y + (label.h - dim_height) / 2,
                    dimension.width,
                    dimension.height,
                ),
                TextAlign::Center => (
                    label.x + (label.w - dim_width) / 2,
                    label.y + (label.h - dim_height) / 2,
                    dimension.width,
                    dimension.height,
                ),
            };

            let rect = Rect::new(x as i32, y as i32, w as u32, h as u32);

            canvas.copy(&texture, None, Some(rect))?;
        }

        canvas.present();

        Ok(())
    }
}

fn ellipsize_text(font: &Font, mut text: String, max_width: usize) -> Result<String, FontError> {
    let (w, _) = font.size_of(&text)?;

    if (w as usize) < max_width {
        return Ok(text);
    }

    loop {
        text.pop();
        text += ELLIPSIS;

        let (w, _) = font.size_of(&text)?;

        if (w as usize) < max_width {
            return Ok(text);
        }

        // pop the ellipsis we just added off
        text.pop();
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
