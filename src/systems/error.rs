#[macro_export]
macro_rules! need_components {
    ($system:ty, $($component:ty),*) => {
        $crate::systems::error::ComponentError {
            need_components: vec![$(stringify!($component).into()),*],
            system_name: stringify!($system).into()
        }
    };
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    ComponentError(#[from] ComponentError),
}

#[derive(Debug, thiserror::Error)]
#[error(
    "Error in system {}: not such components in entity: {:?}",
    system_name,
    need_components
)]
pub struct ComponentError {
    pub need_components: Vec<String>,
    pub system_name: String,
}
