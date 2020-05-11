use grid::TILE_SIZE;
use mergui::Context;
use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::{Color, Graphics},
    lifecycle::{run, EventStream, Settings, Window},
    mint::Vector2,
    Result, Timer,
};
use screens::{game::Game, screen::Screen};

mod character;
mod grid;
mod screens;

pub struct Wrapper<'a> {
    pub window: Window,
    pub gfx: Graphics,
    pub events: EventStream,
    pub context: Context<'a>,
    pub last_cursor_pos: Vector2<f32>,
}
impl<'a> Wrapper<'a> {
    pub fn get_pos_vector(&self, x: f32, y: f32) -> Vector {
        let res = self.window.size();
        Vector::new(x * res.x, y * res.y)
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
                wrapper.gfx.present(&wrapper.window)?;
            }
        }
        // And then we'd do updates and drawing here
        // When this loop ends, the window will close and the application will stop
        // If the window is closed, our application will receive a close event and terminate also
    }
}
