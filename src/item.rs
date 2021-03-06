use crate::graphics::{TextureAtlas, TextureRegion2D};
use cgmath::Point3;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameItem {
    Iron,
    Copper,
}

impl GameItem {
    pub fn iter() -> core::slice::Iter<'static, GameItem> {
        static VARIANTS: [GameItem; 2] = [GameItem::Iron, GameItem::Copper];

        VARIANTS.iter()
    }

    pub fn asteroid_info() -> Vec<(GameItem, Point3<f32>)> {
        vec![
            (GameItem::Iron, Point3::new(0.15, 0.0, 0.0)),
            (GameItem::Copper, Point3::new(0.15, -0.08, -0.2)),
        ]
    }
}

pub fn load_item_icons(atlas: &mut TextureAtlas) -> HashMap<GameItem, TextureRegion2D> {
    let mut map = HashMap::new();
    for item in GameItem::iter() {
        let path = format!(
            "assets/items/{}",
            match item {
                GameItem::Iron => "iron.png",
                GameItem::Copper => "copper.png",
            }
        );
        map.insert(*item, atlas.load_texture(&path));
    }
    map
}

pub struct Inventory {
    items: HashMap<GameItem, u32>,
}

impl Inventory {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let items: HashMap<GameItem, u32> = GameItem::iter()
            .map(|item| (*item, rng.gen_range(10..15)))
            .collect();

        Inventory { items }
    }

    pub fn change_amount(&mut self, item: GameItem, delta: u32) {
        self.items.entry(item).and_modify(|amount| *amount += delta);
    }

    pub fn amount(&self, item: &GameItem) -> u32 {
        *self
            .items
            .get(item)
            .expect("Item is a not a valid variant!")
    }
}
