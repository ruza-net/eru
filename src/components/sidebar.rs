use crate::components::general::Tooltip;

use crate::styles::container;



pub struct Sidebar {
    width: u16,
    tools: Vec<Tooltip<Message>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FreshWorkspace,
}

impl Default for Message {
    fn default() -> Self {
        todo![]
    }
}



impl Default for Sidebar {
    fn default() -> Self {
        let mut new_workspace = Tooltip::from_file("res/img/plus");
        new_workspace.on_press(Message::FreshWorkspace);

        Self {
            width: style::WIDTH,
            tools: vec![new_workspace],
        }
    }
}

impl Sidebar {
    pub fn view(&mut self) -> iced::Element<Message> {
        let width = self.width;

        let tools = self.tools
            .iter_mut()
            .map(|tool| tool.view(Some(width)))
            .collect();

        iced::Container::new(iced::Column::with_children(tools))
            .width(iced::Length::Units(self.width + 2 * container::PADDING))
            .height(iced::Length::Fill)
            .style(style::Default)
            .padding(container::PADDING)
            .into()
    }
}

pub mod style {
    use iced::container;


    pub const WIDTH: u16 = 32;
    pub const BG_SHADE: f32 = 230.;

    pub struct Default;

    impl container::StyleSheet for Default {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(color![ BG_SHADE, BG_SHADE, BG_SHADE ].into()),

                ..fill![]
            }
        }
    }
}
