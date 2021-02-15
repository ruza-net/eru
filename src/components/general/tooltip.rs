use iced::button;

use crate::behavior::SimpleView;
use crate::model::Icon;

use std::path::PathBuf;



// const PADDING: f32 = 8.0;
// const ICON_SIZE: Size = Size::new(32., 32.);


pub struct Tooltip<Msg> {
    icon: Icon,
    on_press: Option<Msg>,
    state: button::State,
}


impl<Msg> Tooltip<Msg> {
    pub fn from_file(res_path: impl Into<PathBuf>) -> Self {
        let icon = Icon::from_file(res_path.into());

        Self {
            icon,

            on_press: None,
            state: button::State::new(),
        }
    }

    pub fn from(el: Box<dyn SimpleView>, color: impl Into<iced::Color>) -> Self {
        let icon = Icon::from(el, color);

        Self {
            icon,

            on_press: None,
            state: button::State::new(),
        }
    }

    pub fn from_text(text: impl ToString, color: impl Into<iced::Color>) -> Self {
        let icon = Icon::from_text(text.to_string(), color);

        Self {
            icon,

            on_press: None,
            state: button::State::new(),
        }
    }

    pub fn on_press(&mut self, msg: Msg) -> &mut Self {
        self.on_press = Some(msg);
        self
    }

    pub fn color_mut(&mut self) -> &mut iced::Color {
        &mut self.icon.color
    }
}

impl<Msg> Tooltip<Msg> where Msg: 'static + Clone + Default {
    pub fn view(&mut self, size: Option<u16>) -> iced::Element<Msg> {
        let mut btn = iced::Button::new(
                &mut self.state,

                self.icon.view(size)
                    .map(|_| fill![]),
            )
            .style(crate::styles::container::Style::cell(self.icon.color));

        if let Some(size) = size {
            btn = btn
                .height(iced::Length::Units(size))
                .width(iced::Length::Units(size));
        }

        if let Some(on_press) = &self.on_press {
            btn = btn.on_press(on_press.clone());
        }

        btn.into()
    }
}



mod style {
    use iced::Color;
    use iced::button::{ Style, StyleSheet };

    use crate::styles::container::button;


    pub(super) struct Border(pub Color);

    impl StyleSheet for Border {
        fn active(&self) -> Style {
            Style {
                shadow_offset: [0., 0.].into(),

                text_color: self.0,
                background: None,

                border_radius: button::RADIUS,
                border_width: button::WIDTH,
                border_color: self.0,
            }
        }

        fn hovered(&self) -> Style {
            self.active()
        }
    }
}
