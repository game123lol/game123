use serde::Deserialize;

/// Компонент, содержащий историю событий от лица сущности, с которой они происходили.
/// События записаны в текстовом представлении, отделены переносом строки
#[derive(Debug)]
pub struct Log(pub String);

impl Log {
    pub fn write(&mut self, event: &str) {
        self.0.push_str((event.to_owned() + "\n").as_str());
    }
}

/// Компонент, означающий, что сущность с этим компонентом - как-либо действующиее
/// существо. Это может быть игрок или неигровой персонаж.
#[derive(Deserialize, Debug)]
pub struct Mob;
