#[macro_export]
macro_rules! fill {
    () => ( std::default::Default::default() );
}

#[macro_export]
macro_rules! accessors {
    ( $($name:ident : $typ:ty),* $(,)? ) => {
        $(
            pub fn $name(mut self, $name: $typ) -> Self {
                self.$name = $name;
                self
            }
        )*
    };
}

#[macro_export]
macro_rules! extract {
    ( $val:expr => $($x:ident),* in $p:pat ) => {
        if let $p = $val { ($($x),*) } else { unreachable![] }
    };
}

#[macro_export]
macro_rules! fn_is_before {
    ( $conv:ident ) => {
        pub fn is_before(&self, before: &ViewIndex, after: &ViewIndex) -> bool {
            let before = self.$conv(before).unwrap();
            let after = self.$conv(after).unwrap();

            self.cells.is_before(before, after).unwrap()
        }
    };
}


#[macro_export]
macro_rules! color {
    ( $r:expr , $g:expr , $b:expr ) => {
        iced::Color {
            r: $r as f32 / 255.,
            g: $g as f32 / 255.,
            b: $b as f32 / 255.,
            a: 1.,
        }
    };

    ( $r:expr , $g:expr , $b:expr , $a:expr ) => {
        iced::Color {
            r: $r as f32 / 255.,
            g: $g as f32 / 255.,
            b: $b as f32 / 255.,
            a: $a as f32,
        }
    };
}

pub fn color_scale_up(color: [f32; 4]) -> [f32; 4] {
    let [r, g, b, a] = color;

    [
        r * 255.,
        g * 255.,
        b * 255.,
        a,
    ]
}
pub fn color_scale_down(color: [f32; 4]) -> [f32; 4] {
    let [r, g, b, a] = color;

    [
        r / 255.,
        g / 255.,
        b / 255.,
        a,
    ]
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



pub trait EncapsulateIter: Iterator {
    fn encapsulate(self) -> <Vec<Self::Item> as IntoIterator>::IntoIter where Self: Sized {
        self.collect::<Vec<_>>()
            .into_iter()
    }
}
impl<I> EncapsulateIter for I where I: Iterator {}

pub trait ProjectIter<X, Y> {
    fn proj_l(self) -> <Vec<X> as IntoIterator>::IntoIter;

    fn proj_r(self) -> <Vec<Y> as IntoIterator>::IntoIter;
}

impl<X, Y, I> ProjectIter<X, Y> for I where I: Iterator<Item = (X, Y)> {
    fn proj_l(self) -> <Vec<X> as IntoIterator>::IntoIter {
        self.map(|(x, _)| x).encapsulate()
    }

    fn proj_r(self) -> <Vec<Y> as IntoIterator>::IntoIter {
        self.map(|(_, y)| y).encapsulate()
    }
}
