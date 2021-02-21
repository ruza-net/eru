use crate::components::general::Tooltip;

use crate::styles::container;



pub struct Sidebar {
    width: u16,
    tools: Vec<Tooltip<Message>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Extrude,
    Sprout,
    Pass,
}

impl Default for Message {
    fn default() -> Self {
        todo![]
    }
}



impl Default for Sidebar {
    fn default() -> Self {
        let mut extrude = Tooltip::from_file("res/img/plus");
        extrude.on_press(Message::Extrude);

        let mut sprout = Tooltip::from_file("res/img/plus");
        *sprout.color_mut() = color![207, 205, 14];
        sprout.on_press(Message::Sprout);

        let mut pass = Tooltip::from_file("res/img/plus");
        *pass.color_mut() = color![29, 129, 179];
        pass.on_press(Message::Pass);

        Self {
            width: style::WIDTH,
            tools: vec![extrude, sprout, pass],
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
