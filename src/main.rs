use grid::TILE_SIZE;
use mergui::{Context, MFont};
use quicksilver::{
    geom::{Rectangle, Transform, Vector},
    graphics::{Color, Graphics, Image, VectorFont},
    lifecycle::{run, EventStream, Settings, Window},
    mint::Vector2,
    Result, Timer,
};
use screens::{game::Game, screen::Screen};
use std::collections::HashMap;

mod character;
mod grid;
mod panel;
mod screens;

pub struct Wrapper<'a> {
    pub window: Window,
    pub gfx: Graphics,
    pub events: EventStream,
    pub context: Context<'a>,
    pub last_cursor_pos: Vector2<f32>,
    pub loaded_font: Option<VectorFont>,
    pub loaded_images: HashMap<String, Image>,
}
impl<'a> Wrapper<'a> {
    pub fn get_pos_vector(&self, x: f32, y: f32) -> Vector {
        let res = self.window.size();
        Vector::new(x * res.x, y * res.y)
    }
    pub async fn get_font(&mut self, size: f32) -> Result<MFont> {
        if let None = &self.loaded_font {
            let font = VectorFont::load("font.ttf").await?;
            self.loaded_font = Some(font);
        }
        let font = self.loaded_font.as_ref().unwrap();
        Ok(MFont::from_font(font, &mut self.gfx, size)?)
    }
    pub async fn get_image(&mut self, path: String) -> Result<Image> {
        match self.loaded_images.entry(path) {
            std::collections::hash_map::Entry::Vacant(x) => {
                let v = Image::load(&mut self.gfx, x.key()).await?;
                Ok(x.insert(v).clone())
            }
            std::collections::hash_map::Entry::Occupied(x) => Ok(x.get().clone()),
        }
    }
}
pub fn grid_pos_to_rectangle(pos: Vector2<i32>) -> Rectangle {
    Rectangle::new(
        (pos.x * TILE_SIZE, pos.y * TILE_SIZE),
        (TILE_SIZE, TILE_SIZE),
    )
}

fn main() {
    run(
        Settings {
            size: [640., 640.].into(),
            title: "Dead wars",
            ..Settings::default()
        },
        app,
    );
}

async fn app(window: Window, gfx: Graphics, events: EventStream) -> Result<()> {
    let context = Context::new([0., 0.].into());
    let mut screen = Game::new().await;
    let mut wrapper = Wrapper {
        window,
        gfx,
        events,
        context,
        last_cursor_pos: [0., 0.].into(),
        loaded_font: None,
        loaded_images: HashMap::new(),
    };
    wrapper.gfx.clear(Color::BLACK);
    wrapper.gfx.present(&wrapper.window)?;
    let mut has_focus = true;
    let mut draw_timer = Timer::time_per_second(60.);
    let mut update_timer = Timer::time_per_second(20.);
    loop {
        while let Some(event) = wrapper.events.next_event().await {
            if let quicksilver::lifecycle::Event::FocusChanged(x) = &event {
                has_focus = x.is_focused();
            }
            if has_focus {
                wrapper.context.event(&event, &wrapper.window);
                if let quicksilver::lifecycle::Event::PointerMoved(x) = &event {
                    wrapper.last_cursor_pos = x.location();
                }
                screen.event(&mut wrapper, &event).await?;
            }
        }
        if has_focus {
            if update_timer.exhaust().is_some() {
                screen.update(&mut wrapper).await?;
            }
            if draw_timer.exhaust().is_some() {
                wrapper.gfx.clear(Color::BLACK);
                screen.draw(&mut wrapper).await?;
                wrapper.gfx.flush(None)?;
                let translation = screen.translate;
                //wrapper.gfx.set_transform(Transform::IDENTITY);
                wrapper.context.render(&mut wrapper.gfx, &wrapper.window)?;
                wrapper.gfx.present(&wrapper.window)?;
            }
        }
    }
}
