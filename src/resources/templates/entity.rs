use std::{collections::HashMap, fs, path::Path};

use hecs::EntityBuilder;
use serde::Deserialize;

use crate::{
    hasher,
    resources::{BodyTemplates, ComponentTemplates},
    GameHasher,
};

#[derive(Debug, Deserialize)]
struct EntityTemplate {
    #[serde(flatten)]
    components: ComponentTemplates,
}

#[derive(Debug, Deserialize)]
pub struct EntityTemplates {
    #[serde(flatten)]
    templates: HashMap<String, EntityTemplate, GameHasher>,
}

//TODO убери анврапы, сделай норм ошибки через thiserror
impl EntityTemplates {
    pub fn new() -> Self {
        Self {
            templates: HashMap::with_hasher(hasher()),
        }
    }
    pub fn load(&mut self, data_path: &Path) -> &mut Self {
        let file = fs::read_to_string(data_path.join("entity_templates.yaml")).unwrap();
        let new: EntityTemplates = serde_yaml::from_str(&file).unwrap();
        self.templates.extend(new.templates);
        self
    }
    //TODO билдеры создаются каждый раз, их надо б как нибудь кэшировать чтоль
    pub fn template(&self, body_templates: &BodyTemplates, template_name: &str) -> EntityBuilder {
        self.templates
            .get(template_name)
            .unwrap()
            .components
            .to_entity_builder(body_templates)
    }
}
