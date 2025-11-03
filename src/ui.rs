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

use crate::{
    hasher, items::Item, Direction, GameHasher, InventoryAction, LogAction, PlayerWorldAction,
    Statistics, UIAction,
};

pub enum UIState {
    Inventory { state: MenuState<Item> },
    Log { text: String },
}

pub type DialogKeys = HashMap<char, UIAction, GameHasher>;

pub struct UIConfig {
    pub dialogs_keys: HashMap<String, DialogKeys, GameHasher>,
    pub world_keys: HashMap<char, PlayerWorldAction, GameHasher>,
}

impl UIConfig {
    pub fn default() -> Self {
        let mut dialogs_keys = HashMap::with_hasher(hasher());
        let mut inventory_keys = HashMap::with_hasher(hasher());
        inventory_keys.insert('q', UIAction::InventoryAction(InventoryAction::Close));
        inventory_keys.insert('d', UIAction::InventoryAction(InventoryAction::DropItem));
        inventory_keys.insert('e', UIAction::InventoryAction(InventoryAction::Equip));
        inventory_keys.insert('u', UIAction::InventoryAction(InventoryAction::Dequip));
        inventory_keys.insert('h', UIAction::Move(Direction::Left));
        inventory_keys.insert('j', UIAction::Move(Direction::Back));
        inventory_keys.insert('k', UIAction::Move(Direction::Forward));
        inventory_keys.insert('l', UIAction::Move(Direction::Right));

        dialogs_keys.insert("inventory".into(), inventory_keys);
        let mut log_keys = HashMap::with_hasher(hasher());
        log_keys.insert('q', UIAction::LogAction(LogAction::Close));
        dialogs_keys.insert("log".into(), log_keys);

        let mut world_keys = HashMap::with_hasher(hasher());

        world_keys.insert('h', PlayerWorldAction::Move(Direction::Left));
        world_keys.insert('j', PlayerWorldAction::Move(Direction::Back));
        world_keys.insert('k', PlayerWorldAction::Move(Direction::Forward));
        world_keys.insert('l', PlayerWorldAction::Move(Direction::Right));
        world_keys.insert('u', PlayerWorldAction::Move(Direction::Up));
        world_keys.insert('n', PlayerWorldAction::Move(Direction::Down));
        world_keys.insert('i', PlayerWorldAction::OpenInventory);
        world_keys.insert('e', PlayerWorldAction::PickUpItem);
        world_keys.insert('p', PlayerWorldAction::OpenLog);
        world_keys.insert('z', PlayerWorldAction::Zoom);
        world_keys.insert('Z', PlayerWorldAction::Unzoom);

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

pub fn inventory(items: &[Item]) {
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

pub struct MenuState<T> {
    pub items: Vec<T>,
    pub pointer: usize,
    pub holded_item: Option<usize>,
}

impl<T> MenuState<T> {
    pub fn new(items: Vec<T>, holded_item: Option<usize>) -> Self {
        Self {
            items,
            holded_item,
            pointer: 0,
        }
    }
    pub fn next_item(&mut self) {
        self.pointer = if self.pointer < self.items.len() - 1 {
            self.pointer + 1
        } else {
            0
        }
    }
    pub fn prev_item(&mut self) {
        self.pointer = if self.pointer > 0 {
            self.pointer - 1
        } else {
            self.items.len() - 1
        }
    }
}

impl<T> Default for MenuState<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            pointer: 0,
            holded_item: None,
        }
    }
}

pub fn menu<T, F: Fn(&T, &mut Ui, bool, bool)>(state: &MenuState<T>, f: F) {
    dialog(|ui| {
        for (n, i) in state.items.iter().enumerate() {
            widgets::Group::new(hash!(), Vec2::new(200., 100.))
                .position(vec2(0., n as f32 * 100.))
                .ui(ui, |x| {
                    f(
                        i,
                        x,
                        n == (state.pointer), //TODO //FIXME КАК ЭТО ВЫШЛО!!!???? УДАЛИТЬ НАХУЙ
                        state.holded_item.is_some_and(|x| x == n),
                    );
                });
        }
    })
}

pub fn hands_menu(state: &MenuState<Item>) {
    menu(state, |item, ui, is_selected, is_holded| {
        widgets::Group::new(hash!(), vec2(200., 100.))
            .position(vec2(0., 0.))
            .ui(ui, |ui| {
                let select_symbol = if is_holded {
                    '*'
                } else if is_selected {
                    '>'
                } else {
                    ' '
                };
                widgets::Label::new(format!("{}{}", select_symbol, item.name.as_str()))
                    .position(vec2(0., 0.))
                    .ui(ui);
            });
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
