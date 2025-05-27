use crate::items::Item;

#[derive(Clone, Debug)]
pub struct Inventory {
    pub items: Vec<Item>,
    pub holded: Option<usize>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            holded: None,
        }
    }
}
