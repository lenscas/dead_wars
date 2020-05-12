use crate::Wrapper;
use mergui::{
    channels::{BasicClickable, Clickable},
    widgets::{button::Button, ButtonConfig},
    Context, FontStyle, LayerId, MFont, Response,
};
use quicksilver::{
    geom::{Rectangle, Shape, Transform, Vector},
    graphics::{Color, Image},
    lifecycle::{Event, MouseButton},
    Result,
};

pub struct Panel<T> {
    widgets: Vec<(T, ButtonConfig)>,
}

pub struct PanelConfig<T> {
    pub options: Vec<(String, T)>,
    pub font: MFont,
    pub top_left: Vector,
    pub width: f32,
    pub background: Image,
    pub text_size: f32,
}

impl<T> Panel<T> {
    pub fn new(config: PanelConfig<T>) -> Self {
        let PanelConfig {
            options,
            font,
            top_left,
            width,
            background,
            text_size,
        } = config;
        let widgets = options
            .into_iter()
            .enumerate()
            .map(|(key, v)| (key, v.0, v.1))
            .map(|(key, text, ret)| {
                let height = top_left.y + (key as f32 * text_size);
                let font_style = FontStyle {
                    font: font.clone(),
                    location: Vector::new(top_left.x, height + text_size / 2.),
                    color: Color::BLACK,
                };
                (
                    ret,
                    ButtonConfig {
                        text,
                        font_style,
                        background: background.clone(),
                        background_location: dbg!(Rectangle::new(
                            (top_left.x, height),
                            (width, text_size),
                        )),
                        blend_color: None,
                        hover_color: None,
                    },
                )
            })
            .collect::<Vec<_>>();
        Self { widgets }
    }
    pub fn event(&mut self, wrapper: &mut Wrapper, event: &Event) -> Option<&T> {
        if let Event::PointerInput(x) = event {
            if x.is_down() && x.button() == MouseButton::Left {
                for (ret, config) in self.widgets.iter() {
                    if config.background_location.contains(wrapper.last_cursor_pos) {
                        return Some(ret);
                    }
                }
            }
        }
        None
    }
    pub fn draw(&self, translate: Vector, wrapper: &mut Wrapper) -> Result<()> {
        for (_, config) in &self.widgets {
            let translated =
                Transform::translate(translate).inverse() * config.background_location.pos;
            let new_rec = Rectangle::new(translated, config.background_location.size);
            wrapper.gfx.draw_image(&config.background, new_rec);
            let new_style = FontStyle {
                location: Transform::translate(translate).inverse() * config.font_style.location,
                ..config.font_style.clone()
            };
            new_style.draw(&mut wrapper.gfx, &config.text)?;
        }
        Ok(())
    }
}
