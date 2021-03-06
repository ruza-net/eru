use crate::components::general::Tooltip;

use crate::styles::container;
use crate::model::Render;



pub struct Sidebar {
    width: u16,
    tools: Vec<Option<Tooltip<Message>>>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Message {
    Enclose,
    Sprout,

    Pass,
    Cut,
    Rename,

    Save,
    Load,
}

impl Default for Message {
    fn default() -> Self {
        unreachable![]
    }
}



macro_rules! tools {
    ( $($lowercase:ident >> $uppercase:ident),* $(,)? ) => {{
        $(
            let mut $lowercase = Tooltip::from_file(format!["res/img/{}", stringify![$lowercase]]);
            $lowercase.on_press(Message::$uppercase);
        )*

        Self {
            width: style::WIDTH,
            tools: vec![ $(Some($lowercase)),* ],
        }
    }};
}

impl Default for Sidebar {
    fn default() -> Self {
        let mut this =
        tools! {
            enclose >> Enclose,
            sprout >> Sprout,
            pass >> Pass,

            cut >> Cut,
            rename >> Rename,

            save >> Save,
            load >> Load,
        };

        this.tools.insert(5, None);
        this.tools.insert(3, None);

        this
    }
}

impl Sidebar {
    pub fn view(&mut self, render: Render) -> iced::Element<Message> {
        let width = self.width;

        let tools = self.tools
            .iter_mut()
            .map(|tool|
                if let Some(tool) = tool {
                    tool.view(Some(width), render)

                } else {
                    iced::Space::with_height((2 * container::PADDING).into()).into()
                }
            )
            .collect();

        iced::Container::new(
                iced::Column::with_children(tools)
                    .spacing(container::PADDING)
            )
            .width(iced::Length::Units(self.width + 2 * container::PADDING))
            .height(iced::Length::FillPortion(1))
            .style(style::Default)
            .padding(container::PADDING)
            .into()
    }
}

pub mod style {
    use iced::container;


    #[cfg(not(feature = "glow"))]
    pub const WIDTH: u16 = 32;

    #[cfg(feature = "glow")]
    pub const WIDTH: u16 = 128;

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
