use iced::container;


pub mod button {
    pub const WIDTH: f32 = 3.;
    pub const RADIUS: f32 = 10.;
}

pub mod cell {
    pub const WIDTH: f32 = 3.;
    pub const RADIUS: f32 = 4.;
}

pub mod color {
    pub const DESATURATE_PERCENT: f64 = 0.85;
    pub const LIGHTEN_PERCENT: f64 = 3.0;
}

pub const PADDING: u16 = 8;

pub const CELL: Style = Style {
    kind: Kind::Cell,
    color: iced::Color::BLACK,
};


pub struct Style {
    kind: Kind,
    color: iced::Color,
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    Cell,
}


impl Style {
    pub fn cell(color: iced::Color) -> Self {
        Self {
            kind: Kind::Cell,
            color,
        }
    }

    fn bleak_color(&self) -> iced::Color {
        let mut color = self.color;
        color.a -= 0.75;

        color
    }

    fn lighten_color(&self) -> iced::Color {
        let linear = self.color.into_linear();

        let mut hsl: colorsys::Hsl = colorsys::Rgb::from(crate::utils::color_scale_up(linear)).into();

        let l = hsl.lightness();
        hsl.set_lightness(l * color::LIGHTEN_PERCENT);

        let s = hsl.saturation();
        hsl.set_saturation(l * color::DESATURATE_PERCENT);

        let linear = crate::utils::color_scale_down(colorsys::Rgb::from(hsl).into());

        linear.into()
    }
}

impl container::StyleSheet for Style {
    fn style(&self) -> container::Style {
        match self.kind {
            Kind::Cell => container::Style {
                border_color: self.color,
                border_width: cell::WIDTH,
                border_radius: cell::RADIUS,

                background: Some({
                    if self.color == iced::Color::BLACK {
                        iced::Color::WHITE.into()

                    } else {
                        self.lighten_color().into()
                    }
                }),

                ..fill![]
            },
        }
    }
}

impl iced::button::StyleSheet for Style {
    fn active(&self) -> iced::button::Style {
        match self.kind {
            Kind::Cell => iced::button::Style {
                shadow_offset: [0., 0.].into(),
                background: Some({
                    if self.color == iced::Color::BLACK {
                        iced::Color::WHITE.into()

                    } else {
                        self.lighten_color().into()
                    }
                }),

                border_radius: cell::RADIUS,
                border_width: cell::WIDTH,
                border_color: self.color,

                text_color: self.color,
            },
        }
    }

    fn hovered(&self) -> iced::button::Style {
        let mut style = self.active();

        style.shadow_offset = [0., 1.].into();

        style
    }
}
