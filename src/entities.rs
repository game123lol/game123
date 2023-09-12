use std::collections::HashSet;

use tetra::{graphics::Rectangle, math::Vec2};

#[derive(Debug)]
pub struct Renderable(pub String, pub Rectangle);

pub struct Player;

pub struct Mob;

pub struct Position(pub Vec2<i32>);

pub struct Item;

pub struct Sight(pub HashSet<(i32, i32)>);

pub struct Name(pub String);
