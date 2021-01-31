use iced::{
    Command,
    Application,
};

use crate::components::{
    sidebar::{ self, Sidebar },
};


// #[derive(Default)]
pub struct App {
    sidebar: Sidebar,
}

#[derive(Debug, Clone)]
pub enum GlobalMessage {
    Sidebar(sidebar::Message),
}


impl Default for App {
    fn default() -> Self {
        Self {
            sidebar: fill![],
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
            GlobalMessage::Sidebar(msg) => match msg {
                sidebar::Message::FreshWorkspace => {},
            },
        }

        Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let sidebar = self.sidebar.view().map(GlobalMessage::Sidebar);

        iced::Row::new()
            .push(sidebar)
            .into()
    }
}
