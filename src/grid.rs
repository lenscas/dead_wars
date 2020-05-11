use crate::{
    character::{CharacterContainer, CharacterType},
    Wrapper,
};
use quicksilver::{geom::Rectangle, graphics::Color, Result as quickResult};
use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Type {
    Water,
    Grass,
    Moutain,
    Road,
}

impl TryFrom<char> for Type {
    type Error = ();
    fn try_from(value: char) -> Result<Self, ()> {
        let value = value.to_ascii_lowercase();
        Ok(match value {
            'w' => Type::Water,
            'g' => Type::Grass,
            'm' => Type::Moutain,
            'r' => Type::Road,
            x => panic!("Character {} is not a valid conversion", x),
        })
    }
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
    pub fn new(width: usize, height: usize, json_map: &ParseableMap) -> Self {
        let mut grid = Vec::new();
        for y in 0..width {
            let tiles = json_map.tiles.get(y).cloned();
            let tiles = dbg!(tiles);
            let tiles = tiles
                .unwrap_or_else(|| panic!("could not get {}", y))
                .chars()
                .collect::<Vec<_>>();
            for x in 0..height {
                let tile = tiles
                    .get(x)
                    .cloned()
                    .map(Type::try_from)
                    .expect(&format!("could not get {} from {}", y, x))
                    .expect("something has gone wrong");
                grid.push((x as i32, y as i32, tile));
            }
        }
        Self {
            grid,
            height: height as i32,
            width: width as i32,
        }
    }
}

#[derive(Deserialize)]
pub struct ParseableCharacter {
    pub x: i32,
    pub y: i32,
    pub char_type: CharacterType,
}
#[derive(Deserialize)]
pub struct ParseableMap {
    pub tiles: Vec<String>,
    pub characters: Vec<ParseableCharacter>,
}
impl ParseableMap {
    pub fn parse(self) -> quickResult<(Grid, CharacterContainer)> {
        let height = self.tiles.len();
        let width = self.tiles.first().expect("map is empty").len();
        let grid = Grid::new(height, width, &self);
        let characters = CharacterContainer::new(&self);
        Ok((grid, characters))
    }
}
