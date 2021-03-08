use crate::components::general::Tooltip;

use crate::styles::container;



pub struct Sidebar {
    width: u16,
    tools: Vec<Tooltip<Message>>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Message {
    Enclose,
    Pass,
}

impl Default for Message {
    fn default() -> Self {
        unreachable![]
    }
}



impl Default for Sidebar {
    fn default() -> Self {
        let mut enclose = Tooltip::from_file("res/img/plus");

        enclose.label("Enclose".to_string());
        enclose.on_press(Message::Enclose);


        let mut sprout = Tooltip::from_file("res/img/plus");
        *sprout.color_mut() = color![230, 188, 65];

        sprout.label("Sprout".to_string());
        sprout.on_press(Message::Sprout);


        let mut pass = Tooltip::from_file("res/img/plus");
        *pass.color_mut() = color![29, 129, 179];

        pass.label("Pass".to_string());
        pass.on_press(Message::Pass);

        Self {
            width: style::WIDTH,
            tools: vec![enclose, sprout, pass],
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
