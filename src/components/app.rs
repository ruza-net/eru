use std::fmt;


use iced::{
    Command,
    Application,
};

use crate::components::{
    opetope::{ self, Diagram },

    general::{
        sidebar,
        main_layout::{ self, State, Layout },
    },
};



pub type Data = opetope::data::Selectable<String>;


pub struct App {
    opetope: Diagram<Data>,

    layout: Layout,
}

pub enum Error {
    Opetope(opetope::Error),

    EmptyName,
}
impl From<opetope::Error> for Error {
    fn from(e: opetope::Error) -> Self {
        Self::Opetope(e)
    }
}

#[derive(Debug, Clone)]
pub enum GlobalMessage {
    Sidebar(sidebar::Message),
    Opetope(opetope::Message),

    Layout(main_layout::Message),

    FocusNext,

    Ticked,
    Idle,
}
impl Default for GlobalMessage {
    fn default() -> Self {
        Self::Idle
    }
}


impl Default for App {
    fn default() -> Self {
        let opetope = opetope::Tower::init("0".to_string().into()).1.into_next().unwrap();

        Self {
            layout: fill![],

            opetope,
        }
    }
}



impl App {
    fn extrude(&mut self, name: Data, wrap: Data) {
        if name.is_empty() || wrap.is_empty() {
            self.error(Error::EmptyName)

        } else if let Some(sel) = self.opetope.selected_cells() {
            match
            self.opetope
                .extrude(&sel, name, wrap)
                .ok()
            {
                Ok(_) => {},

                Err(e) => self.error(e.into()),
            }
        }
    }

    fn split(&mut self, name: Data, wrap_top: Data, wrap_bot: Data) {
        if name.is_empty() || wrap_bot.is_empty() || wrap_top.is_empty() {
            self.error(Error::EmptyName)

        } else if let Some(sel) = self.opetope.selected_cells() {
            match
            self.opetope
                .split(&sel, name, wrap_top, wrap_bot)
                .ok()
            {
                Ok(_) => {},

                Err(e) => self.error(e.into()),
            }
        }
    }

    fn sprout(&mut self, data: Vec<(opetope::ViewIndex, Data, Data)>) {
        if data.iter().any(|(_, name, wrap)| name.is_empty() || wrap.is_empty()) {
            self.error(Error::EmptyName)

        } else if let Some(sel) = self.opetope.selected_cells() {
            for (cell, name, wrap) in data {
                match
                self.opetope
                    .sprout(&cell, name.into(), wrap.into())
                    .ok()
                {
                    Ok(_) => {},

                    Err(e) => self.error(e.into()),
                }
            }
        }
    }

    fn prepare_rename(&mut self) {
        if let Some(sel) = self.opetope.selected_cells() {
            let mut old_names = vec![];

            for cell in sel.as_cells() {
                old_names.push(self.opetope.cell(&cell).unwrap().data().inner().clone());
            }

            self.layout.state = State::rename(old_names);
        }
    }

    fn rename(&mut self, new_names: Vec<String>) {
        let sel = self.opetope.selected_cells().unwrap();

        for (cell, new_name) in sel.as_cells().iter().zip(new_names) {
            match self.opetope.rename(cell, new_name.into()) {
                Ok(_) => {},

                Err(e) => self.error(e.into()),
            }
        }
    }

    // TODO: Custom workspace dirs
    //
    fn save(&self) {
        use std::fs::File;
        use std::io::Write;
        use serde_json::ser;

        let mut f = File::create("/Users/honza/opetope.json").unwrap();

        f.write(ser::to_string(&self.opetope).unwrap().as_bytes()).unwrap();
    }

    // TODO: Custom workspace dirs
    //
    fn load(&mut self) {
        use std::fs::File;
        use std::io::Read;
        use serde_json::de;

        let mut f = File::open("/Users/honza/opetope.json").unwrap();

        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();

        self.opetope = de::from_str(&buf).unwrap();// TODO: Make this not panic, but `error`
    }

    fn error(&mut self, e: Error) {
        self.layout.error(e);
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

    fn update(&mut self, message: Self::Message, _: &mut iced::Clipboard) -> Command<Self::Message> {
        match message {
            GlobalMessage::Idle => {},

            GlobalMessage::Sidebar(msg) =>
                match msg {
                    sidebar::Message::Pass => {
                        let groups_left: Vec<_> =
                        self.opetope
                            .iter_groups()
                            .map(|(face, cell)| (face, cell.clone()))
                            .collect();

                        if groups_left.is_empty() {
                            self.opetope.into_next(vec![]).unwrap();

                        } else {
                            self.layout.state = State::pass(groups_left);
                        }
                    },

                    sidebar::Message::Enclose =>
                        if let Some(sel) = self.opetope.selected_cells() {
                            if self.opetope.is_at_bottom(&sel).unwrap() {
                                self.layout.state = State::extrude();

                            } else {
                                self.layout.state = State::split();
                            }
                        },

                    sidebar::Message::Sprout => {
                        self.layout.state = State::sprout();

                        match &mut self.layout.state {
                            State::ProvideSprout { ref mut ends, .. } =>
                                if let Some(sel) = self.opetope.selected_cells() {
                                    for cell in sel.as_cells() {
                                        let end = self.opetope.cell(&cell).unwrap();

                                        ends.push((cell, end, None));
                                    }
                                } else {
                                    self.layout.state = State::default();// TODO: Warning
                                },

                            _ =>
                                unreachable![],
                        }
                    },

                    // TODO: Make rename tooltip

                    sidebar::Message::Save =>
                        self.save(),

                    sidebar::Message::Load =>
                        self.load(),
                },


            GlobalMessage::Layout(msg) =>
                match msg {
                    main_layout::Message::UpdatedName(new_name) =>
                        match &mut self.layout.state {
                            State::Default | State::ProvidePass { .. } =>
                                unreachable![],

                            State::Rename { remaining, .. } =>
                                remaining.last_mut().unwrap().value = new_name,

                            State::ProvideExtrude { name, .. } =>
                                name.value = new_name,

                            State::ProvideSprout { last_end, .. } =>
                                last_end.value = new_name,

                            State::ProvideSplit { name, .. } =>
                                name.value = new_name,
                        }

                    main_layout::Message::UpdatedFirstWrap(new_name) =>
                        match &mut self.layout.state {
                            State::Default | State::Rename { .. } =>
                                unreachable![],

                            State::ProvideExtrude { wrap, .. } =>
                                wrap.value = new_name,

                            State::ProvideSprout { last_wrap, .. } =>
                                last_wrap.value = new_name,

                            State::ProvideSplit { wrap_top, .. } =>
                                wrap_top.value = new_name,
                            
                            State::ProvidePass { last, .. } =>
                                last.value = new_name,
                        }

                    main_layout::Message::UpdatedSecondWrap(new_name) =>
                        match &mut self.layout.state {
                            State::Default | State::ProvidePass { .. } | State::ProvideSprout { .. } | State::Rename { .. } =>
                                unreachable![],

                            State::ProvideExtrude { wrap, .. } =>
                                wrap.value = new_name,

                            State::ProvideSplit { wrap_bot, .. } =>
                                wrap_bot.value = new_name,
                        }

                    main_layout::Message::ConfirmPopUp =>
                        match self.layout.state.take() {// TODO: Reject empty names
                            State::Default =>
                                self.prepare_rename(),

                            State::Rename { mut remaining, mut renamed, pop_up } => {
                                let last = remaining.pop().unwrap();

                                renamed.push(last.value);

                                if !remaining.is_empty() {
                                    self.layout.state = State::Rename { remaining, renamed, pop_up };

                                } else {
                                    self.rename(renamed);
                                }
                            },

                            State::ProvideExtrude { name, wrap, .. } => {
                                let name = name.value.into();
                                let wrap = wrap.value.into();

                                self.extrude(name, wrap)
                            },

                            State::ProvideSprout { mut wraps, mut ends, last_end, last_wrap, pop_up } => {
                                    assert![wraps.len() < ends.len(), "sprout saturated on `update`"];

                                    ends[wraps.len()].2 = Some(last_end.value.into());
                                    wraps.push(last_wrap.value.into());

                                    if wraps.len() == ends.len() {
                                        self.sprout(
                                            ends.into_iter()
                                                .zip(wraps)
                                                .map(|((index, _, sprout), wrap)| (
                                                    index,
                                                    sprout.unwrap(),
                                                    wrap,
                                                ))
                                                .collect()
                                        );

                                    } else {
                                        let last_end = fill![];
                                        let last_wrap = fill![];

                                        self.layout.state = State::ProvideSprout { ends, wraps, last_end, last_wrap, pop_up };
                                    }
                                },

                            State::ProvideSplit { name, wrap_top, wrap_bot, .. } => {
                                let name = name.value.into();
                                let wrap_top = wrap_top.value.into();
                                let wrap_bot = wrap_bot.value.into();

                                self.split(name, wrap_top, wrap_bot)
                            },

                            State::ProvidePass { mut groups_left, mut wraps, last, pop_up } => {
                                    assert![!groups_left.is_empty(), "pass saturated on `update`"];

                                    let (face, _) = groups_left.pop().unwrap();

                                    wraps.push(opetope::MetaCell {
                                        data: last.value.into(),
                                        face,
                                    });

                                    if groups_left.is_empty() {
                                        match self.opetope.into_next(wraps) {
                                            Ok(_) => {},

                                            Err(e) =>
                                                self.error(e.into()),
                                        }

                                    } else {
                                        let last = fill![];

                                        self.layout.state = State::ProvidePass { groups_left, wraps, last, pop_up };
                                    }
                                },
                        },

                    main_layout::Message::ExitPopUp => {
                        self.layout.state.take();
                    },
                },

            GlobalMessage::Opetope(msg) =>
                match msg {
                    opetope::Message::Idle => unreachable!["idle message"],

                    opetope::Message::Select(cell) => {
                        self.opetope.select(&cell).unwrap();
                    },
                },

            GlobalMessage::FocusNext =>
                match &mut self.layout.state {
                    State::ProvideExtrude { name, wrap, .. } | State::ProvideSprout { last_end: name, last_wrap: wrap, .. } =>
                        if name.state.is_focused() {
                            name.state.unfocus();
                            wrap.state.focus();

                        } else {
                            wrap.state.unfocus();
                            name.state.focus();
                        },

                    State::ProvideSplit { name, wrap_top, wrap_bot, .. } =>
                        if name.state.is_focused() {
                            name.state.unfocus();
                            wrap_top.state.focus();

                        } else if wrap_top.state.is_focused() {
                            wrap_top.state.unfocus();
                            wrap_bot.state.focus();

                        } else {
                            wrap_bot.state.unfocus();
                            name.state.focus();
                        },

                    State::ProvidePass { last, .. } =>
                        last.state.focus(),

                    State::Rename { remaining, .. } =>
                        remaining.last_mut().unwrap().state.focus(),

                    _ => {},
                },
            
            GlobalMessage::Ticked =>
                self.layout.tick(),
        }

        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let not_editing = matches![self.layout.state, State::Default];

        iced::Subscription::batch(vec![
            iced_native::subscription::events_with(move |e, _| {
                use iced_native::Event;

                match e {
                    Event::Keyboard(key) =>
                        match key {
                            iced::keyboard::Event::KeyPressed { key_code, .. } =>
                                match key_code {
                                    iced::keyboard::KeyCode::Tab =>
                                        Some(GlobalMessage::FocusNext),

                                    iced::keyboard::KeyCode::Enter =>
                                        Some(GlobalMessage::Layout(main_layout::Message::ConfirmPopUp)),

                                    iced::keyboard::KeyCode::Escape =>
                                        Some(GlobalMessage::Layout(main_layout::Message::ExitPopUp)),

                                    iced::keyboard::KeyCode::E if not_editing =>
                                        Some(GlobalMessage::Sidebar(sidebar::Message::Enclose)),

                                    iced::keyboard::KeyCode::S if not_editing =>
                                        Some(GlobalMessage::Sidebar(sidebar::Message::Sprout)),

                                    iced::keyboard::KeyCode::N if not_editing =>
                                        Some(GlobalMessage::Sidebar(sidebar::Message::Pass)),

                                    _ =>
                                        None,
                                },

                            _ =>
                                None,
                        },

                    _ =>
                        None,
                }
            }),
            iced::time::every(std::time::Duration::from_secs(1))
                .map(|_| GlobalMessage::Ticked),
        ])
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        self.layout.view(&mut self.opetope)
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
impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Opetope(e) => write![fmt, "{}", e],
            
            Self::EmptyName => write![fmt, "cell name cannot be empty"],
        }
    }
}
