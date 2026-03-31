use iced::widget::space;
use iced::{Element, Task};

pub struct SettingsState {}

pub enum SettingsMsg {}

impl SettingsState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, msg: SettingsMsg) -> Task<SettingsMsg> {
        Task::none()
    }

    pub fn view(&'_ self) -> Element<'_, SettingsMsg> {
        space().into()
    }
}
