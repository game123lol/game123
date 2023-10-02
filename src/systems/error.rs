use std::fmt::Display;

#[macro_export]
macro_rules! need_components {
    ($system:ty, $($component:ty),*) => {
        crate::systems::error::Error {
            need_components: vec![$(stringify!($component).into()),*],
            system_name: stringify!($system).into()
        }
    };
}

#[derive(Debug)]
pub struct Error {
    pub need_components: Vec<String>,
    pub system_name: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let comps_string = match self.need_components.len() {
            0 => panic!("Try generate components_need error without components"),
            1 => format!("{} component", self.need_components[0].clone()),
            2 => format!(
                "{} and {} components",
                self.need_components[0], self.need_components[1]
            ),
            _ => {
                let mut str = self.need_components[0].clone();
                for i in self.need_components[1..self.need_components.len() - 2].iter() {
                    str.push_str(&format!(", {}", i))
                }
                format!(
                    "{} and {} components",
                    str,
                    self.need_components.last().unwrap().clone()
                )
            }
        };
        write!(
            f,
            "Can't run {} without entity with {}",
            self.system_name, comps_string
        )
    }
}

impl std::error::Error for Error {}
