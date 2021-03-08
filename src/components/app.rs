use std::fmt;


use iced::{
    Command,
    Application,
};

use crate::components::{
    sidebar::{ self, Sidebar },
    opetope::{ self, Diagram },
};



const ERR_DURATION: u64 = 5;

pub struct App {
    sidebar: Sidebar,

    count: usize,
    opetope: Diagram<opetope::data::Selectable<String>>,

    error_countdown: u64,
    last_error: Option<opetope::Error>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum GlobalMessage {
    Sidebar(sidebar::Message),
    Opetope(opetope::Message),

    Ticked,
}


impl Default for App {
    fn default() -> Self {
        let opetope = opetope::Tower::init("0".to_string().into()).1.into_next().unwrap();

        Self {
            sidebar: fill![],

            opetope,

            count: 1,

            last_error: None,
            error_countdown: 0,
        }
    }
}



impl App {
    fn enclose(&mut self) {
        let name = self.count.to_string();
        let wrap = name.clone() + "_wrap";

        if let Some(sel) = self.opetope.selected_cells() {
            match
            self.opetope
                .extrude(&sel, name.clone().into(), wrap.clone().into())
                .ok()
            {
                Ok(_) =>
                    self.count += 1,

                Err(_) =>
                    match
                    self.opetope
                        .split(&sel, name.into(), (wrap.clone() + "_0").into(), (wrap + "_1").into())
                        .ok()
                    {
                        Ok(_) =>
                            self.count += 1,

                        Err(e) =>
                            self.error(e),
                    },
            }

        }
    }

    fn sprout(&mut self) {
        if let Some(sel) = self.opetope.selected_cells() {
            for cell in sel.as_cells() {
                let name = self.count.to_string();
                let wrap = name.clone() + "_wrap";

                match
                self.opetope
                    .sprout(&cell, name.into(), wrap.into())
                    .ok()
                {
                    Ok(_) =>
                        self.count += 1,

                    Err(e) =>
                        self.error(e),
                }
            }
        }
    }

    fn error(&mut self, e: opetope::Error) {
        self.last_error = Some(e);
        self.error_countdown = ERR_DURATION;
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
                sidebar::Message::Enclose =>
                    self.enclose(),

                sidebar::Message::Sprout =>
                    self.sprout(),

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

            GlobalMessage::Ticked =>
                if self.error_countdown > 0 {
                    self.error_countdown -= 1;
                }
        }

        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(std::time::Duration::from_secs(1))
            .map(|_| GlobalMessage::Ticked)
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let sidebar = self.sidebar.view().map(GlobalMessage::Sidebar);
        let opetope = self.opetope.view().map(GlobalMessage::Opetope);

        let main =
        iced::Row::new()
            .push(sidebar)
            .push(opetope);


        if self.error_countdown > 0 {
            let err_text = self.last_error.as_ref().unwrap().to_string();

            let err_msg = iced_aw::Badge::new(iced::Text::new(err_text))
                .style(iced_aw::style::badge::Danger);

            iced::Column::new()
                .push(err_msg)
                .push(main)
                .into()

        } else {
            main.into()
        }
    }
}



impl fmt::Display for opetope::Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            opetope::Error::CannotSproutGroup(_cell) =>
                write![fmt, "Cannot sprout group"],

            opetope::Error::CannotGroupDisconnected(_sel) =>
                write![fmt, "Cells are not connected"],

            opetope::Error::CellsDoNotFormTree(_sel) =>
                write![fmt, "Cells do not form a tree"],

            opetope::Error::CannotConvertAlreadyGrouped =>
                write![fmt, "Cannot pass to next level when the layer already has groups"],


            // Internal
            //
            opetope::Error::IndexError(e) =>
                write![fmt, "INTERNAL: error while indexing: {:?}", e],

            opetope::Error::TooMuchDepth(depth) =>
                write![fmt, "INTERNAL: too much depth: {}", depth],

            opetope::Error::NoSuchCell(cell) =>
                write![fmt, "INTERNAL: cell does not exist: {}", cell],

            opetope::Error::NoCellWithInputs(inputs) =>
                write![fmt, "INTERNAL: no cell with inputs: {:?}", inputs],

            opetope::Error::CannotSplitBoundaryCells(sel) =>
                write![fmt, "INTERNAL: cannot split boundary cells: {:?}", sel],

            opetope::Error::CannotExtrudeNestedCells(sel) =>
                write![fmt, "INTERNAL: cannot extrude nested cells: {:?}", sel],

        }
    }
}
