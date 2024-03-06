use std::sync::Arc;

use tetra::math::Vec2;

/// Компонент, который должен быть у сущностей, которые будут иметь позицию на
/// игровой карте. Это может быть, например, лежащий на земле предмет, игрок или неигровой персонаж.
#[derive(PartialEq, Clone, Copy)]
pub struct Position(pub Vec2<i32>);

/// Компонент, имя какой-либо сущности.
pub struct Name(pub Arc<str>);
