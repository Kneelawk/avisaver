use crate::app::icons;
use iced::widget::tooltip::Position;
use iced::widget::{button, column, container, row, rule, space, svg, tooltip};
use iced::{Border, Element, Task};
use iced::border::Radius;

pub struct RootState {
    selected_tab: SelectedTab,
}

#[derive(Clone)]
pub enum RootMsg {
    SetTab(SelectedTab),
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum SelectedTab {
    #[default]
    Saves,
    Templates,
    Settings,
}

impl RootState {
    pub fn new() -> Self {
        Self {
            selected_tab: Default::default(),
        }
    }

    pub fn update(&mut self, msg: RootMsg) -> Task<RootMsg> {
        match msg {
            RootMsg::SetTab(tab) => {
                self.selected_tab = tab;
                Task::none()
            }
        }
    }

    pub fn view(&'_ self) -> Element<'_, RootMsg> {
        row![
            column![
                side_button(self.selected_tab, SelectedTab::Saves),
                side_button(self.selected_tab, SelectedTab::Templates),
                space::vertical(),
                side_button(self.selected_tab, SelectedTab::Settings),
            ],
            rule::vertical(1),
            container("test"),
        ]
        .into()
    }
}

fn side_button(selected_tab: SelectedTab, for_tab: SelectedTab) -> Element<'static, RootMsg> {
    let (icon, tooltip_name) = match for_tab {
        SelectedTab::Saves => (icons::PERSON.clone(), "Saves"),
        SelectedTab::Templates => (icons::GROUP.clone(), "Templates"),
        SelectedTab::Settings => (icons::SETTINGS.clone(), "Settings"),
    };

    let mut button = button(svg(icon))
        .style(move |theme, status| {
            let style = if selected_tab == for_tab {
                button::primary(theme, status)
            } else {
                button::background(theme, status)
            };

            button::Style {
                border: Border {
                    radius: Radius::new(128.0),
                    ..style.border
                },
                ..style
            }
        })
        .padding(5.0)
        .width(64.0)
        .height(64.0);

    if selected_tab != for_tab {
        button = button.on_press(RootMsg::SetTab(for_tab));
    }

    tooltip(button, tooltip_name, Position::Right).into()
}
