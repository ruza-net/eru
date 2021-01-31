#[macro_use]
mod utils;
mod styles;
mod behavior;

mod model;


use iced::Application;



pub type Color = [f32; 4];


fn main() {
    components::App::run(fill![]).expect("error running application");
}
