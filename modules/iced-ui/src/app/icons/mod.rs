use iced::widget::svg::Handle;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ADD: Handle = Handle::from_memory(include_bytes!("mui-add.svg"));
    pub static ref GROUP: Handle = Handle::from_memory(include_bytes!("mui-group.svg"));
    pub static ref PERSON: Handle = Handle::from_memory(include_bytes!("mui-person.svg"));
    pub static ref REMOVE: Handle = Handle::from_memory(include_bytes!("mui-remove.svg"));
    pub static ref SETTINGS: Handle = Handle::from_memory(include_bytes!("mui-settings.svg"));
}
