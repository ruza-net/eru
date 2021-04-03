#![allow(dead_code)]

use iced::{ button, tooltip };

use crate::behavior::SimpleView;
use crate::model::{ Icon, Render };

use std::path::PathBuf;



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

    pub fn from(el: Box<dyn SimpleView>, label: Option<String>, color: impl Into<iced::Color>) -> Self {
        let icon = Icon::from(el, label, color);

        Self {
            icon,

            on_press: None,
            state: button::State::new(),
        }
    }

    pub fn from_text(text: impl ToString, label: Option<String>, color: impl Into<iced::Color>) -> Self {
        let icon = Icon::from_text(text.to_string(), label, color);

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

    pub fn label(&mut self, label: String) -> &mut Self {
        self.icon.label = Some(label);
        self
    }

    pub fn color_mut(&mut self) -> &mut iced::Color {
        &mut self.icon.color
    }
}

macro_rules! view {
    ( $fn_name:ident $([$($lt:lifetime)? $mt:ident])? ( $self:ident $(, $arg:ident : $typ:ty)* $(,)? ) >> $state:expr, $size:ident, $render:ident ) => {
        pub fn $fn_name<'s>(& $( $($lt)? $mt)? $self $(, $arg: $typ)* ) -> iced::Element<'s, Msg> {
            let mut btn = iced::Button::new(
                    $state,

                    $self.icon.view($size)
                        .map(|_| fill![]),
                )
                .style(crate::styles::container::Style::cell($self.icon.color));

            if let Some(size) = $size {
                btn = btn
                    .height(iced::Length::Units(size))
                    .width(iced::Length::Units(size));
            }

            if $render == Render::Interactive {
                if let Some(on_press) = &$self.on_press {
                    btn = btn.on_press(on_press.clone());
                }

                if let Some(label) = &$self.icon.label {
                    tooltip::Tooltip::new(btn, label, tooltip::Position::FollowCursor)
                    .style(crate::styles::container::Tooltip)
                    .into()

                } else {
                    btn.into()
                }

            } else {
                btn.into()
            }
        }
    };
}
impl<Msg> Tooltip<Msg> where Msg: 'static + Clone + Default {
    view! {
        view['s mut](self, size: Option<u16>, render: Render)
        >> &mut self.state, size, render
    }

    view! {
        view_with_state(self, state: &'s mut button::State, size: Option<u16>, render: Render)
        >> state, size, render
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
