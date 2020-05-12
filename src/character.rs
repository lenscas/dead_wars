use crate::{grid::ParseableMap, grid_pos_to_rectangle, Wrapper};
use quicksilver::{geom::Vector, graphics::Color, mint::Vector2, Result};
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
#[derive(Deserialize, Clone, Copy)]
pub enum CharacterType {
    Basic,
}
impl CharacterType {
    pub fn get_range(&self) -> i32 {
        match self {
            CharacterType::Basic => 3,
        }
    }
}

pub struct Character {
    pub position: Vector2<i32>,
    pub id: u64,
    pub char_type: CharacterType,
}
impl Character {
    pub fn new(id: u64, position: Vector2<i32>, char_type: CharacterType) -> Self {
        Self {
            position,
            id,
            char_type,
        }
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
    last_move: Option<(u64, Vector2<i32>)>,
}
impl CharacterContainer {
    pub fn new(map: &ParseableMap) -> Self {
        let mut next_id = 0;

        let mut characters = HashMap::new();
        for character in &map.characters {
            let loc = [character.x, character.y].into();
            characters.insert(next_id, Character::new(next_id, loc, character.char_type));
            next_id += 1
        }
        Self {
            characters,
            next_id,
            path: None,
            last_move: None,
        }
    }

    pub fn finalize(&mut self) {
        self.last_move = None;
    }

    pub fn undo(&mut self) {
        if let Some((id, loc)) = self.last_move {
            if let Some(v) = self.characters.get_mut(&id) {
                v.position = loc
            };
        }
        self.finalize();
    }

    pub fn get_char_ids_in_range_of(&self, id: u64) -> Vec<(u64, Vector2<i32>)> {
        let res = self
            .characters
            .get(&id)
            .map(|v| (v.position, v.char_type.get_range()));
        if let Some((position, range)) = res {
            let pos_as_vec = Vector::new(position.x, position.y);
            self.characters
                .iter()
                .map(|(id, character)| (*id, character))
                .filter(|(check_id, character)| {
                    id != *check_id
                        && (Vector::new(character.position.x, character.position.y)
                            .distance(pos_as_vec)
                            .ceil()
                            < (range as f32))
                })
                .map(|(id, character)| (id, character.position))
                .collect()
        } else {
            Vec::new()
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
            return;
        }
        self.last_move = Some((
            id,
            self.characters
                .get(&id)
                .expect(&format!("id {} not found", id))
                .position,
        ));
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

    pub fn update(&mut self) -> Result<bool> {
        if let Some((id, path)) = &mut self.path {
            let next = path.pop_front();
            if let Some(next) = next {
                let char = self
                    .characters
                    .get_mut(id)
                    .expect(&format!("id not valid? {}", id));
                char.position = next;
                Ok(false)
            } else {
                self.path = None;
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}
