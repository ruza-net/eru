use iced::container;

pub mod button {
    pub const WIDTH: f32 = 3.;
    pub const RADIUS: f32 = 10.;
}

pub mod cell {
    pub const WIDTH: f32 = 3.;
    pub const RADIUS: f32 = 4.;

    pub const SPACING: u16 = 15;
}

pub mod color {
    pub const DESATURATE_PERCENT: f64 = 0.85;
    pub const LIGHTEN_PERCENT: f64 = 3.0;

    pub const SELECTED: iced::Color = color![255; 154; 97];
}

pub const PADDING: u16 = 8;

pub const LINE: Style = Style {
    kind: Kind::Line,
    color: iced::Color::BLACK,
};

pub const CELL: Style = Style {
    kind: Kind::Cell { selected: false },
    color: iced::Color::BLACK,
};

pub const SELECTED_CELL: Style = Style {
    kind: Kind::Cell { selected: true },
    color: iced::Color::BLACK,
};

pub struct Style {
    kind: Kind,
    color: iced::Color,
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    Line,
    Cell { selected: bool },
}

impl Style {
    pub fn cell(color: iced::Color) -> Self {
        Self {
            kind: Kind::Cell { selected: false },
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

        let mut hsl: colorsys::Hsl =
            colorsys::Rgb::from(crate::utils::color_scale_up(linear)).into();

        let l = hsl.lightness();
        hsl.set_lightness(l * color::LIGHTEN_PERCENT);

        let s = hsl.saturation();
        hsl.set_saturation(s * color::DESATURATE_PERCENT);

        let linear = crate::utils::color_scale_down(colorsys::Rgb::from(hsl).into());

        linear.into()
    }
}

impl container::StyleSheet for Style {
    fn style(&self) -> container::Style {
        match self.kind {
            Kind::Cell { selected } => container::Style {
                border_color: self.color,
                border_width: cell::WIDTH,
                border_radius: cell::RADIUS,

                background: Some({
                    if selected {
                        color::SELECTED.into()

                    } else if self.color == iced::Color::BLACK {
                        iced::Color::WHITE.into()

                    } else {
                        self.lighten_color().into()
                    }
                }),

                ..fill![]
            },

            Kind::Line => container::Style {
                background: Some(self.color.into()),

                ..fill![]
            },
        }
    }
}

impl iced::button::StyleSheet for Style {
    fn active(&self) -> iced::button::Style {
        match self.kind {
            Kind::Cell { selected } => iced::button::Style {
                shadow_offset: [0., 0.].into(),
                background: Some({
                    if selected {
                        color::SELECTED.into()

                    } else if self.color == iced::Color::BLACK {
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

            Kind::Line => iced::button::Style {
                background: Some(self.color.into()),

                ..fill![]
            },
        }
    }

    fn hovered(&self) -> iced::button::Style {
        let mut style = self.active();

        style.shadow_offset = [0., 1.].into();

        style
    }
}
