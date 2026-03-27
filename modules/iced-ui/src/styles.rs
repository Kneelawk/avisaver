use iced::widget::button;
use iced::{Border, Theme};

pub fn menu_button(theme: &Theme, status: button::Status) -> button::Style {
    let base = button::background(theme, status);

    button::Style {
        border: Border::default(),
        ..base
    }
}
