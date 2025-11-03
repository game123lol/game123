use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use serde::Deserialize;

use crate::{hasher, GameHasher, Property};

#[derive(Debug, Deserialize)]
pub struct ItemTemplates {
    #[serde(flatten)]
    pub templates: HashMap<String, ItemTemplate, GameHasher>,
}

#[derive(Debug, Deserialize)]
pub struct ItemTemplate {
    #[serde(rename = "sprite")]
    pub sprite_name: String,
    #[serde(default)]
    pub properties: HashMap<String, Property, GameHasher>,
    #[serde(default)]
    pub flags: HashSet<String>,
}

impl ItemTemplates {
    pub fn new() -> Self {
        Self {
            templates: HashMap::with_hasher(hasher()),
        }
    }
    pub fn load(&mut self, data_path: &Path) -> &mut Self {
        let file = fs::read_to_string(data_path.join("item_templates.yaml")).unwrap();
        let new: Self = serde_yaml::from_str(&file).unwrap();
        self.templates.extend(new.templates);
        self
    }
}
