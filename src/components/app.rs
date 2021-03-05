use iced::{
    Command,
    Application,
};

use crate::components::{
    sidebar::{ self, Sidebar },
    opetope::{ self, Diagram },
};



pub struct App {
    sidebar: Sidebar,

    count: usize,
    opetope: Diagram<opetope::data::Selectable<String>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum GlobalMessage {
    Sidebar(sidebar::Message),
    Opetope(opetope::Message),
}


impl Default for App {
    fn default() -> Self {
        let opetope = opetope::Tower::init("0".to_string().into()).1.into_next().unwrap();

        Self {
            sidebar: fill![],

            opetope,

            count: 1,
        }
    }
}



impl App {
    fn enclose(&mut self) -> Result<(), opetope::Error> {
        let mut name = self.count.to_string();
        let mut wrap = name.clone() + "_wrap";

        if let Some(sel) = self.opetope.selected_cells() {
            match
            self.opetope
                .extrude(&sel, name.into(), wrap.into())
                .ok()
            {
                Ok(_) => {
                    self.count += 1;

                    Ok(())
                },

                Err(_) => {
                    for cell in sel.as_cells() {
                        name = self.count.to_string();
                        wrap = name.clone() + "_wrap";

                        self.opetope
                            .sprout(&cell, name.into(), wrap.into())
                            .ok()?;

                        self.count += 1;
                    }

                    Ok(())
                },
            }

        } else {
            Ok(())
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
                sidebar::Message::Enclose => {
                    self.enclose().unwrap()
                },

                sidebar::Message::Pass => {
                    take_mut::take(&mut self.opetope, |op| op.into_next().unwrap());
                },
            },

            GlobalMessage::Opetope(msg) => match msg {
                opetope::Message::Idle => unreachable!["idle message"],

                opetope::Message::Select(cell) => {
                    self.opetope.select(&cell).unwrap();
                },
            },
        }

        Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let sidebar = self.sidebar.view().map(GlobalMessage::Sidebar);
        let opetope = self.opetope.view().map(GlobalMessage::Opetope);

        iced::Row::new()
            .push(sidebar)
            .push(opetope)
            .into()
    }
}
