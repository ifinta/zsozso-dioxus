pub mod home;
pub mod networking;
pub mod info;
pub mod settings;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Tab {
    #[default]
    Home,
    Networking,
    Info,
    Settings,
}
