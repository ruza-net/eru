#[macro_use]
mod utils;
mod styles;
mod behavior;
mod components;

mod model;


use iced::Application;



fn main() {
    components::App::run(fill![]).expect("error running application");
}
