use crate::{grid::ParseableMap, grid_pos_to_rectangle, Wrapper};
use quicksilver::{graphics::Color, mint::Vector2, Result};
use std::collections::{HashMap, VecDeque};
pub struct Character {
    pub position: Vector2<i32>,
    pub id: u64,
}
impl Character {
    pub fn new(id: u64, position: Vector2<i32>) -> Self {
        Self { position, id }
    }
    pub fn draw(&self, wrapper: &mut Wrapper<'_>) -> Result<()> {
        wrapper
            .gfx
            .fill_rect(&grid_pos_to_rectangle(self.position), Color::RED);
        Ok(())
    }
}

pub struct CharacterContainer {
    pub next_id: u64,
    pub characters: HashMap<u64, Character>,
    pub path: Option<(u64, VecDeque<Vector2<i32>>)>,
}
impl CharacterContainer {
    pub fn new(map: &ParseableMap) -> Self {
        let mut next_id = 0;

        let mut characters = HashMap::new();
        for char_loc in &map.characters {
            characters.insert(
                next_id,
                Character::new(next_id, Vector2::<i32>::from(*char_loc)),
            );
            next_id += 1
        }
        Self {
            characters,
            next_id: next_id,
            path: None,
        }
    }
    pub fn get_char_id_by_pos(&self, position: Vector2<i32>) -> Option<u64> {
        self.characters
            .iter()
            .find(|(_, character)| character.position == position)
            .map(|(v, _)| *v)
    }
    pub fn move_character(&mut self, id: u64, path: Vec<Vector2<i32>>) {
        if let Some(x) = &self.path {
            dbg!(x);
            return;
        }
        self.path = Some((id, path.into_iter().collect()))
    }
    pub fn is_moving(&self) -> bool {
        self.path.is_some()
    }
    pub fn draw(&self, wrapper: &mut Wrapper<'_>) -> Result<()> {
        for (_, character) in &self.characters {
            character.draw(wrapper)?;
        }
        Ok(())
    }
    pub fn update(&mut self) -> Result<()> {
        if let Some((id, path)) = &mut self.path {
            let next = path.pop_front();
            if let Some(next) = next {
                let char = self
                    .characters
                    .get_mut(id)
                    .expect(&format!("id not valid? {}", id));
                char.position = next;
            } else {
                self.path = None;
            }
        }
        Ok(())
    }
}
