use std::sync::Arc;

use vek::Vec3;

/// Компонент, который должен быть у сущностей, которые будут иметь позицию на
/// игровой карте. Это может быть, например, лежащий на земле предмет, игрок или неигровой персонаж.
#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub struct Position(pub Vec3<i32>);

impl Position {
    pub fn new(pos_x: i32, pos_y: i32, pos_z: i32) -> Self {
        Position(Vec3::new(pos_x, pos_y, pos_z))
    }
}

/// Компонент, имя какой-либо сущности.
pub struct Name(pub Arc<str>);
