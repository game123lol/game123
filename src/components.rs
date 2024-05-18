use std::sync::Arc;

use vek::Vec3;

/// Компонент, который должен быть у сущностей, которые будут иметь позицию на
/// игровой карте. Это может быть, например, лежащий на земле предмет, игрок или неигровой персонаж.
#[derive(Eq, PartialEq, Clone, Copy, Hash)]
pub struct Position(pub Vec3<i32>);

/// Компонент, имя какой-либо сущности.
pub struct Name(pub Arc<str>);
