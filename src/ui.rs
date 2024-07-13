use std::collections::HashMap;

use macroquad::{
    miniquad::window::screen_size,
    prelude::{vec2, Color, Vec2},
    text::load_ttf_font,
    ui::{
        hash, root_ui,
        widgets::{self},
        Skin, Ui,
    },
};

use crate::{hasher, items::Item, Direction, GameHasher, PlayerAction, Statistics};

pub enum UIState {
    No,
    Inventory { items: Vec<Item> },
    Log { text: String },
    Debug,
}

pub type DialogKeys = HashMap<char, PlayerAction, GameHasher>;

pub struct UIConfig {
    pub dialogs_keys: HashMap<String, DialogKeys, GameHasher>,
    pub world_keys: HashMap<char, PlayerAction, GameHasher>,
}

impl UIConfig {
    pub fn default() -> Self {
        let mut dialogs_keys = HashMap::with_hasher(hasher());
        let mut inventory_keys = HashMap::with_hasher(hasher());
        inventory_keys.insert('q', PlayerAction::CloseInventory);
        dialogs_keys.insert("inventory".into(), inventory_keys);
        let mut log_keys = HashMap::with_hasher(hasher());
        log_keys.insert('q', PlayerAction::CloseLog);
        dialogs_keys.insert("log".into(), log_keys);

        let mut world_keys = HashMap::with_hasher(hasher());

        world_keys.insert('h', PlayerAction::Move(Direction::Left));
        world_keys.insert('j', PlayerAction::Move(Direction::Back));
        world_keys.insert('k', PlayerAction::Move(Direction::Forward));
        world_keys.insert('l', PlayerAction::Move(Direction::Right));
        world_keys.insert('u', PlayerAction::Move(Direction::Up));
        world_keys.insert('n', PlayerAction::Move(Direction::Down));
        world_keys.insert('i', PlayerAction::OpenInventory);
        world_keys.insert('e', PlayerAction::PickUpItem);
        world_keys.insert('p', PlayerAction::OpenLog);
        world_keys.insert('z', PlayerAction::Zoom);
        world_keys.insert('Z', PlayerAction::Unzoom);

        Self {
            dialogs_keys,
            world_keys,
        }
    }
}

pub async fn set_skin() {
    let mut font = load_ttf_font("assets/Terminus.ttf").await.unwrap();
    font.set_filter(macroquad::texture::FilterMode::Nearest);
    let style = root_ui()
        .style_builder()
        .text_color(Color::from_hex(0xFFFFFF))
        .color(Color::from_hex(0x000000))
        .with_font(&font)
        .unwrap()
        .font_size(14)
        .build();
    let skin = Skin {
        window_style: style.clone(),
        label_style: style.clone(),
        window_titlebar_style: style.clone(),
        ..root_ui().default_skin()
    };
    root_ui().push_skin(&skin);
}

pub fn dialog<F: FnOnce(&mut Ui)>(f: F) {
    let screen_size: Vec2 = screen_size().into();
    widgets::Window::new(
        hash!(),
        vec2(screen_size.x / 5., screen_size.y / 5.),
        vec2(screen_size.x / 5. * 3., screen_size.y / 5. * 3.),
    )
    .titlebar(false)
    .ui(&mut root_ui(), f);
}

pub fn inventory(items: &Vec<Item>) {
    dialog(|ui| {
        for (n, i) in items.iter().enumerate() {
            widgets::Label::new(&i.name)
                .position(vec2(0., n as f32 * 14.))
                .ui(ui);
        }
    });
}

pub fn log(log: &str) {
    dialog(|ui| {
        for (n, line) in log.split('\n').enumerate() {
            widgets::Label::new(line)
                .position(vec2(0., n as f32 * 14.))
                .ui(ui);
        }
    })
}

pub fn debug(stats: &Statistics) {
    widgets::Window::new(hash!(), vec2(0., 0.), vec2(300., 300.))
        .movable(true)
        .label("Statistics")
        .ui(&mut root_ui(), |ui| {
            for (n, line) in stats.show().split('\n').enumerate() {
                widgets::Label::new(line)
                    .position(vec2(0., n as f32 * 14.))
                    .ui(ui);
            }
        });
}
