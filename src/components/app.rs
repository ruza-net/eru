use iced::{
    Command,
    Application,
};



// #[derive(Default)]
pub struct App {
}

#[derive(Debug, Clone)]
pub enum GlobalMessage {
}


impl Default for App {
    fn default() -> Self {
        Self {
        }
    }
}



impl Application for App {
    type Message = GlobalMessage;

    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            fill![],
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "eru".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
        }

        Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
    }
}
