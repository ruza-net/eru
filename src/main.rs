#[macro_use]
mod utils;
mod styles;


use iced::Application;



pub type Color = [f32; 4];


fn main() {
    components::App::run(fill![]).expect("error running application");
}
