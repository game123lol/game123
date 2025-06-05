#[macro_export]
macro_rules! need_components {
    ($system:ty, $($component:ty),*) => {
        $crate::systems::error::Error {
            need_components: vec![$(stringify!($component).into()),*],
            system_name: stringify!($system).into()
        }
    };
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Error in system {}: not such components in entity: {:?}",
    system_name,
    need_components
)]
pub struct Error {
    pub need_components: Vec<String>,
    pub system_name: String,
}
