use super::screen::Screen;
use crate::{
    character::CharacterContainer,
    grid::{Grid, ParseableMap, TILE_SIZE},
    grid_pos_to_rectangle,
    panel::{Panel, PanelConfig},
    Wrapper,
};
use async_trait::async_trait;
use quicksilver::{
    geom::{Rectangle, Transform, Vector},
    graphics::Color,
    lifecycle::{Key, MouseButton},
    load_file,
    mint::Vector2,
};
use std::{collections::HashSet, convert::TryFrom};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum Directions {
    Left,
    Up,
    Right,
    Down,
}

impl Directions {
    pub fn change_x(self) -> bool {
        match self {
            Self::Up | Self::Down => false,
            _ => true,
        }
    }
    pub fn change_y(self) -> bool {
        !self.change_x()
    }
}

impl From<Directions> for Vector {
    fn from(from: Directions) -> Vector {
        match from {
            Directions::Down => (0, -1).into(),
            Directions::Up => (0, 1).into(),
            Directions::Left => (1, 0).into(),
            Directions::Right => (-1, 0).into(),
        }
    }
}
impl TryFrom<Key> for Directions {
    type Error = ();
    fn try_from(k: Key) -> Result<Self, Self::Error> {
        match k {
            Key::W => Ok(Directions::Up),
            Key::A => Ok(Directions::Left),
            Key::S => Ok(Directions::Down),
            Key::D => Ok(Directions::Right),
            _ => Err(()),
        }
    }
}

pub enum AfterMoveOptions {
    Undo,
    Fight,
    Stay,
}
pub enum InputState {
    Normal,
    DrawingPath(u64, Vec<Vector2<i32>>),
    WaitingForCharacterMovement(u64, Vector2<i32>),
    SelectingActionAfterMove(Panel<AfterMoveOptions>, u64, Vector2<i32>),
    SelectingFight(u64, Vector2<i32>, Vec<(u64, Vector2<i32>)>),
}

impl InputState {
    pub fn to_waiting_for_fight(&mut self) -> Result<(u64, Vec<Vector2<i32>>), ()> {
        match self {
            InputState::DrawingPath(id, path) => {
                let id = *id;
                let last_place = path.last().unwrap().clone();
                drop(path);
                let old = std::mem::replace(
                    self,
                    InputState::WaitingForCharacterMovement(id, last_place),
                );
                if let InputState::DrawingPath(id, path) = old {
                    Ok((id, path))
                } else {
                    unreachable!()
                }
            }
            _ => Err(()),
        }
    }
}

pub struct Game {
    pub moving: HashSet<Directions>,
    pub translate: Vector,
    pub grid: Grid,
    pub selected: InputState, //Option<(u64, Vec<Vector2<i32>>)>,
    pub characters: CharacterContainer,
}

impl Game {
    pub async fn new() -> Self {
        let file = load_file("map.json")
            .await
            .expect("something has gone wrong");
        let (grid, characters) = serde_json::from_slice::<ParseableMap>(&file)
            .expect("couldn't parse")
            .parse()
            .expect("gone wrong");
        Self {
            moving: HashSet::new(),
            translate: Vector::new(0, 0),
            grid,
            characters,
            selected: InputState::Normal,
        }
    }
}

impl Game {
    fn cursor_pos_to_grid(&self, pos: Vector2<f32>) -> Vector2<i32> {
        let raw_pos = Transform::translate(self.translate).inverse() * Vector::new(pos.x, pos.y);
        [
            (raw_pos.x / TILE_SIZE as f32).floor() as i32,
            (raw_pos.y / TILE_SIZE as f32).floor() as i32,
        ]
        .into()
    }
}

#[async_trait(?Send)]
impl Screen for Game {
    async fn draw(&mut self, wrapper: &mut crate::Wrapper<'_>) -> quicksilver::Result<()> {
        self.grid.draw(wrapper);
        if let InputState::DrawingPath(_, path) = &self.selected {
            for v in path {
                wrapper.gfx.fill_rect(
                    &Rectangle::new(
                        Vector::new(v.x * TILE_SIZE, v.y * TILE_SIZE),
                        (TILE_SIZE as i32, TILE_SIZE as i32),
                    ),
                    Color::WHITE,
                );
            }
        }
        self.characters.draw(wrapper)?;
        match &self.selected {
            InputState::SelectingFight(_, _, targets) => {
                for (_, target) in targets {
                    wrapper
                        .gfx
                        .fill_rect(&grid_pos_to_rectangle(target.clone()), Color::ORANGE);
                }
            }
            InputState::SelectingActionAfterMove(x, _, _) => x.draw(self.translate, wrapper)?,
            _ => {}
        }

        Ok(())
    }
    async fn update(
        &mut self,
        wrapper: &mut crate::Wrapper<'_>,
    ) -> quicksilver::Result<Option<Box<dyn Screen>>> {
        let mut translate = self.translate.clone();
        self.moving.iter().copied().for_each(|v| {
            translate += Vector::from(v).times(Vector::new(10, 10));
        });
        if translate.x > 0. {
            translate.x = 0.;
        }
        if translate.y > 0. {
            translate.y = 0.
        }
        if self.translate != translate {
            let cursor_pos = self.cursor_pos_to_grid(wrapper.last_cursor_pos);
            if let InputState::DrawingPath(_, path) = &mut self.selected {
                path.push(cursor_pos);
            }
        }
        self.translate = translate;
        wrapper.gfx.set_transform(Transform::translate(translate));
        if self.characters.update()? {
            if let InputState::WaitingForCharacterMovement(id, at) = self.selected {
                self.selected = InputState::SelectingActionAfterMove(
                    Panel::new(PanelConfig {
                        options: vec![
                            ("Fight".into(), AfterMoveOptions::Fight),
                            ("Stay".into(), AfterMoveOptions::Stay),
                            ("Undo".into(), AfterMoveOptions::Undo),
                        ],
                        font: wrapper.get_font(20.).await?,
                        top_left: Vector::new(10., 20.),
                        width: 50.,
                        background: wrapper.get_image("pixel.png".into()).await?,
                        text_size: 30.,
                    }),
                    id,
                    at,
                );
            }
        }
        Ok(None)
    }
    async fn event(
        &mut self,
        wrapper: &mut Wrapper<'_>,
        event: &quicksilver::lifecycle::Event,
    ) -> quicksilver::Result<Option<Box<dyn Screen>>> {
        if let InputState::SelectingActionAfterMove(panel, id, location) = &mut self.selected {
            if let Some(chosen) = panel.event(wrapper, event) {
                match chosen {
                    AfterMoveOptions::Undo => {
                        self.characters.undo();
                        self.selected = InputState::Normal;
                    }
                    AfterMoveOptions::Fight => {
                        let in_range = self.characters.get_char_ids_in_range_of(*id);
                        self.selected = InputState::SelectingFight(*id, *location, in_range);
                    }
                    AfterMoveOptions::Stay => {
                        self.selected = InputState::Normal;
                        self.characters.finalize();
                    }
                }
                return Ok(None);
            }
        }
        match event {
            quicksilver::lifecycle::Event::PointerInput(x) => {
                if x.button() == MouseButton::Left {
                    if x.is_down() {
                        match &self.selected {
                            InputState::DrawingPath(_, _) => {
                                let (id, path) = self.selected.to_waiting_for_fight().unwrap();
                                self.characters.move_character(id, path);
                            }
                            InputState::Normal => {
                                if self.characters.is_moving() {
                                    return Ok(None);
                                }
                                let pos = self.cursor_pos_to_grid(wrapper.last_cursor_pos.clone());
                                if let Some(id) = self.characters.get_char_id_by_pos(pos) {
                                    self.selected = InputState::DrawingPath(id, vec![pos])
                                }
                            }
                            InputState::SelectingFight(_, _, targets) => {
                                let cursor_pos =
                                    self.cursor_pos_to_grid(wrapper.last_cursor_pos.clone());
                                if let Some(_) = targets
                                    .iter()
                                    .find(|(_, loc)| loc == &cursor_pos)
                                    .map(|(id, _)| id)
                                {
                                    self.selected = InputState::Normal
                                }
                            }
                            _ => {}
                        }
                    }
                } else if x.button() == MouseButton::Right && x.is_down() {
                    self.selected = InputState::Normal;
                    self.characters.undo();
                }
            }
            quicksilver::lifecycle::Event::PointerMoved(x) => {
                let loc = x.location();
                let grid_pos = self.cursor_pos_to_grid(loc);
                if let InputState::DrawingPath(_, path) = &mut self.selected {
                    if path.len() > 1
                        && path
                            .get(path.len() - 2)
                            .expect("Selected is not long enough???")
                            == &grid_pos
                    {
                        path.pop();
                    } else if path.last().expect("Path was empty?") != &grid_pos {
                        path.push(grid_pos);
                    }
                }
            }
            quicksilver::lifecycle::Event::KeyboardInput(x) => {
                if x.is_down() {
                    if let Ok(key) = Directions::try_from(x.key()) {
                        self.moving.insert(key);
                    }
                } else {
                    if let Ok(key) = Directions::try_from(x.key()) {
                        self.moving.remove(&key);
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }
}
