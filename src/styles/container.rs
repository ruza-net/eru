use iced::container;


pub mod button {
    pub const WIDTH: f32 = 3.;
    pub const RADIUS: f32 = 10.;
}

pub mod cell {
    pub const WIDTH: f32 = 3.;
    pub const RADIUS: f32 = 4.;
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
                        self.bleak_color().into()
                    }
                }),

                ..Default::default()
            },
        }
    }
}
