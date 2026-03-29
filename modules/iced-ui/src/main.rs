pub mod app;
pub mod utils;

#[macro_use]
extern crate tracing;

use crate::app::ASState;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    info!("Hello AviSaver ^-^");

    iced::daemon(ASState::new, ASState::update, ASState::view)
        .subscription(ASState::subscriptions)
        .theme(ASState::theme)
        .title(ASState::title)
        .run()
}
