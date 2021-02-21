use iced::{
    Command,
    Application,
};

use crate::components::{
    sidebar::{ self, Sidebar },
    opetope::{ self, Diagram },
};


// #[derive(Default)]
pub struct App {
    sidebar: Sidebar,

    count: usize,
    selection: Option<opetope::Selection>,
    opetope: Diagram<opetope::data::Selectable<String>>,
}

#[derive(Debug, Clone)]
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
            selection: None,

            count: 1,
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
        macro_rules! mutate {
            ( $method:ident ( $($arg:expr),* ) ) => ({
                let name = self.count.to_string();
                let wrap = name.clone() + "_wrap";

                self.opetope.$method( $($arg, )* name.into(), wrap.into() ).unwrap();

                self.count += 1;
            });
        }

        match message {
            GlobalMessage::Sidebar(msg) => match msg {
                sidebar::Message::Extrude => {
                    if let Some(sel) = &self.selection {
                        mutate![ extrude(sel) ]
                    }
                },

                sidebar::Message::Sprout => {
                    if let Some(sel) = &self.selection {
                        for cell in sel.as_cells() {
                            mutate![ sprout(&cell) ]
                        }
                    }
                },

                sidebar::Message::Pass => {
                    take_mut::take(&mut self.opetope, |op| op.into_next().unwrap());
                },
            },

            GlobalMessage::Opetope(msg) => match msg {
                opetope::Message::Idle => unreachable!["idle message"],

                opetope::Message::Select(cell) => {
                    self.selection = self.opetope.select(&cell).unwrap();
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
