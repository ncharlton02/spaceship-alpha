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
    pub fn stack(&self, amount: u32) -> ItemStack {
        ItemStack {
            item: *self,
            amount,
        }
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemStack {
    pub item: GameItem,
    pub amount: u32,
}

pub struct Inventory {
    items: HashMap<GameItem, u32>,
}

impl Inventory {
    pub fn new() -> Self {
        let items: HashMap<GameItem, u32> = GameItem::iter().map(|item| (*item, 0)).collect();

        Inventory { items }
    }

    pub fn has_enough_items(&self, item_stack: &ItemStack) -> bool {
        self.amount(&item_stack.item) >= item_stack.amount
    }

    pub fn add(&mut self, item: GameItem, delta: u32) {
        self.items.entry(item).and_modify(|amount| *amount += delta);
    }

    pub fn remove(&mut self, item: GameItem, delta: u32) {
        self.items.entry(item).and_modify(|amount| *amount -= delta);
    }

    pub fn amount(&self, item: &GameItem) -> u32 {
        *self
            .items
            .get(item)
            .expect("Item is a not a valid variant!")
    }
}
