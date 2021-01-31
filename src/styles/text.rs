#![allow(dead_code)]


pub mod fonts {
    macro_rules! font {
        ( $const_name:ident = $font_name:ident : $font_path:expr ) => {
            pub const $const_name: iced::Font = iced::Font::External {
                name: stringify![$font_name],
                bytes: include_bytes![ $font_path ],
            };
        };
    }

    font! { BOLD = Bold: "../../res/fonts/GillSans-SemiBold.ttf" }
    font! { REGULAR = Regular: "../../res/fonts/GillSans-Regular.ttf" }
}


pub mod sizes {
    pub const LARGE: u16 = 25;
    pub const SMALL: u16 = 14;

    pub const NORMAL: u16 = 18;
}


pub mod colors {
    pub const PRIMARY: iced::Color = color![45.; 71.; 149.];
    pub const SECONDARY: iced::Color = color![83.; 98.; 140.];
}
