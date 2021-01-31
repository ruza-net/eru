#[macro_use]
mod utils;


use iced::Application;



pub type Color = [f32; 4];


fn main() {
    components::App::run(fill![]).expect("error running application");
}
