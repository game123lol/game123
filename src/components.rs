use std::collections::HashSet;

use tetra::math::Vec2;

#[derive(Debug)]
pub struct Renderable(pub String);

pub struct Player;

pub struct Mob;

pub struct Position(pub Vec2<i32>);

pub struct Item;

pub struct Sight(pub HashSet<(i32, i32)>);

pub struct Name(pub String);
