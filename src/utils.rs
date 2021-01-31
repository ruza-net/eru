#[macro_export]
macro_rules! fill {
    () => ( std::default::Default::default() );
}

#[macro_export]
macro_rules! color {
    ( $r:expr ; $g:expr ; $b:expr ) => {
        iced::Color {
            r: $r as f32 / 256.,
            g: $g as f32 / 256.,
            b: $b as f32 / 256.,
            a: 1.,
        }
    };

    ( $r:expr ; $g:expr ; $b:expr ; $a:expr ) => {
        iced::Color {
            r: $r as f32 / 256.,
            g: $g as f32 / 256.,
            b: $b as f32 / 256.,
            a: $a as f32 / 256.,
        }
    };
}
