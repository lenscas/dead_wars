use crate::Wrapper;
use quicksilver::{geom::Rectangle, graphics::Color};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Type {
    Water,
    Grass,
    Moutain,
    Road,
}
impl From<Type> for Color {
    fn from(t: Type) -> Self {
        match t {
            Type::Water => Color::BLUE,
            Type::Grass => Color::GREEN,
            Type::Moutain => Color::INDIGO,
            Type::Road => Color::BLACK,
        }
    }
}
pub const TILE_SIZE: i32 = 64;
pub struct Grid {
    grid: Vec<(i32, i32, Type)>,
    height: i32,
    width: i32,
}
impl Grid {
    pub fn draw(&mut self, wrapper: &mut Wrapper<'_>) {
        self.grid.iter().for_each(|(x, y, tile)| {
            wrapper.gfx.fill_rect(
                &Rectangle::new((*x * TILE_SIZE, *y * TILE_SIZE), (TILE_SIZE, TILE_SIZE)),
                (*tile).into(),
            )
        })
    }
    pub fn new(width: i32, height: i32) -> Self {
        let mut grid = Vec::new();
        for y in 0..width {
            for x in 0..height {
                let tile = match (x + y) % 4 {
                    0 => Type::Water,
                    1 => Type::Grass,
                    2 => Type::Moutain,
                    3 => Type::Road,
                    x => panic!("got {} max 3", x),
                };
                grid.push((x, y, tile));
            }
        }
        Self {
            grid,
            height,
            width,
        }
    }
}
