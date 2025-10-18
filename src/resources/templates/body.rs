use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use serde::Deserialize;

use crate::{
    body::{Body, BodyPart, BodyPartPart, BoneGroup, Organ},
    hasher, GameHasher, Property,
};

#[derive(Debug, Deserialize)]
pub struct BodyTemplates {
    #[serde(flatten)]
    pub templates: HashMap<String, BodyTemplate, GameHasher>,
}

#[derive(Debug, Deserialize)]
pub struct BodyTemplate {
    #[serde(flatten)]
    bodyparts: HashMap<String, BodyPartTemplate, GameHasher>,
}

impl BodyTemplate {
    fn new() -> Self {
        Self {
            bodyparts: HashMap::with_hasher(GameHasher::default()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BodyPartTemplate {
    #[serde(flatten)]
    bodypartparts: HashMap<String, BodyPartSegmentTemplate, GameHasher>,
}

impl BodyPartTemplate {
    fn new() -> Self {
        Self {
            bodypartparts: HashMap::with_hasher(GameHasher::default()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct BodyPartSegmentTemplate {
    #[serde(default)]
    organs: Vec<String>,
    #[serde(default)]
    bone_groups: Vec<String>,
    #[serde(default)]
    properties: HashMap<String, Property, GameHasher>,
    #[serde(default)]
    flags: HashSet<String>,
}

impl BodyPartSegmentTemplate {
    fn new() -> Self {
        Self {
            organs: Vec::new(),
            bone_groups: Vec::new(),
            properties: HashMap::with_hasher(GameHasher::default()),
            flags: HashSet::new(),
        }
    }
}

impl BodyTemplates {
    pub fn new() -> Self {
        Self {
            templates: HashMap::with_hasher(hasher()),
        }
    }
    pub fn load(&mut self, data_path: &Path) -> &mut Self {
        let file = fs::read_to_string(data_path.join("body_templates.yaml")).unwrap();
        let new: Self = serde_yaml::from_str(&file).unwrap();
        self.templates.extend(new.templates);
        self
    }
    pub fn template(&self, template_name: &str) -> Body {
        let template = self
            .templates
            .get(template_name)
            .expect("this body template isnt exist");
        let mut body = Body::new();
        for (name, part_template) in &template.bodyparts {
            let mut part = BodyPart::new();
            for (part_name, part_segment_template) in &part_template.bodypartparts {
                let mut part_segment = BodyPartPart::new();
                for i in part_segment_template.organs.iter() {
                    part_segment.add_organ(i.clone(), Organ::new())
                }
                for i in part_segment_template.bone_groups.iter() {
                    part_segment.add_bone_group(i.clone(), BoneGroup::new())
                }
                for i in part_segment_template.properties.iter() {
                    part_segment.add_property(i.0.clone(), i.1.to_owned())
                }
                part.add_part(part_name.clone(), part_segment);
            }
            body.add_part(name.clone(), part);
        }
        body
    }
}
