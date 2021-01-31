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

#[macro_export]
macro_rules! file_contents {
    ( $file:ident >> $buf:ident ) => (
        {
            use std::io::Read;
            use std::fs::File;

            File::open(&$file)
                .expect(&format!["error opening file: {:?}", $file])
            .read_to_string(&mut $buf)
                .expect(&format!["error reading data: {:?}", $file]);
        }
    );
}


pub trait ToOption {
    fn map<X>(&self, f: impl FnOnce() -> X) -> Option<X>;
}
pub trait OrErr {
    fn or_err<E>(&self, e: impl FnOnce() -> E) -> Result<(), E>;
}
pub trait ThenOk {
    fn then_ok<O>(&self, o: impl FnOnce() -> O) -> Result<O, ()>;
}

impl ToOption for bool {
    fn map<X>(&self, f: impl FnOnce() -> X) -> Option<X> {
        if *self {
            Some(f())

        } else {
            None
        }
    }
}
impl OrErr for bool {
    fn or_err<E>(&self, e: impl FnOnce() -> E) -> Result<(), E> {
        if *self {
            Ok(())
        } else {
            Err(e())
        }
    }
}
impl ThenOk for bool {
    fn then_ok<O>(&self, o: impl FnOnce() -> O) -> Result<O, ()> {
        if *self {
            Ok(o())
        } else {
            Err(())
        }
    }
}
