use crate::styles::container::PADDING;

use crate::components::{
    opetope,

    app::{ Error, GlobalMessage, Data },
    pop_up::{ self, PopUp, Form },

    general::Sidebar,
};



#[derive(Debug, Clone)]
pub enum Message {
    UpdatedName(String),
    UpdatedFirstWrap(String),
    UpdatedSecondWrap(String),

    ExitPopUp,
    ConfirmPopUp,
}

const ERR_DURATION: u64 = 5;



#[derive(Default, Debug, Clone)]
pub struct NameSlot {
    pub state: iced::text_input::State,
    pub value: String,
}
impl From<String> for NameSlot {
    fn from(value: String) -> Self {
        Self {
            value,

            ..fill![]
        }
    }
}

#[derive(Debug, Clone)]
pub enum State {
    Default,

    Rename {
        pop_up: pop_up::State,

        remaining: Vec<NameSlot>,
        renamed: Vec<String>,
    },

    ProvideExtrude {
        pop_up: pop_up::State,

        name: NameSlot,
        wrap: NameSlot,
    },

    ProvideSprout {
        pop_up: pop_up::State,

        last_end: NameSlot,
        last_wrap: NameSlot,
        
        wraps: Vec<String>,
        ends: Vec<(opetope::ViewIndex, opetope::Cell<Data>, Option<String>)>,
    },

    ProvideSplit {
        pop_up: pop_up::State,

        name: NameSlot,
        wrap_top: NameSlot,
        wrap_bot: NameSlot,
    },

    ProvidePass {
        pop_up: pop_up::State,

        last: NameSlot,

        wraps: Vec<opetope::MetaCell<Data>>,
        groups_left: Vec<(opetope::Face, opetope::MetaCell<Data>)>,
    },
}
impl Default for State {
    fn default() -> Self {
        Self::Default
    }
}
impl State {
    pub fn extrude() -> Self {
        Self::ProvideExtrude { pop_up: fill![], name: fill![], wrap: fill![] }
    }

    pub fn sprout() -> Self {
        Self::ProvideSprout { pop_up: fill![], last_end: fill![], last_wrap: fill![], ends: fill![], wraps: fill![] }
    }

    pub fn split() -> Self {
        Self::ProvideSplit { pop_up: fill![], name: fill![], wrap_top: fill![], wrap_bot: fill![] }
    }

    pub fn pass(groups_left: Vec<(opetope::Face, opetope::MetaCell<Data>)>) -> Self {
        Self::ProvidePass { pop_up: fill![], last: fill![], wraps: fill![], groups_left }
    }

    pub fn rename(names: Vec<String>) -> Self {
        let remaining = names.into_iter().map(Into::into).collect();

        Self::Rename { pop_up: fill![], remaining, renamed: vec![] }
    }

    pub fn take(&mut self) -> Self {
        let mut ret = None;

        take_mut::take(self, |this| {
            ret = Some(this);

            Self::default()
        });

        ret.unwrap()
    }
}


#[derive(Default)]
pub struct Layout {
    error_countdown: u64,
    error_pop_up: pop_up::State,

    error: Option<Error>,

    pub state: State,
    sidebar: Sidebar,
}



impl Layout {
    pub fn error(&mut self, e: Error) {
        self.error = Some(e);
        self.error_countdown = ERR_DURATION;
    }

    pub fn tick(&mut self) {
        if self.error_countdown > 0 {
            self.error_countdown -= 1;
        }
    }

    pub fn view<'app>(&'app mut self, opetope: &'app mut opetope::Diagram<Data>) -> iced::Element<'app, GlobalMessage> {
        let interact =
        match self.state {
            State::Default => crate::model::Render::Interactive,

            _ => crate::model::Render::Static,
        };

        let sidebar = self.sidebar.view(interact).map(GlobalMessage::Sidebar);// TODO: Max height or portion
        let opetope = opetope.view(interact).map(GlobalMessage::Opetope);

        let opetope =
        iced::Container::new(opetope)
            .padding(PADDING);

        let mut main =
        iced::Row::new()
            .push(sidebar)
            .push(opetope)
            .into();

        if self.error_countdown > 0 {
            let err_text = self.error.as_ref().unwrap().to_string();

            let err_msg = Form::error(err_text);

            main =
            PopUp::new(main, err_msg)
                .location(pop_up::Location::Top)
                .view(&mut self.error_pop_up)
        }

        match &mut self.state {
            State::Default =>
                main,

            State::Rename { pop_up, remaining, .. } =>
                PopUp::new(
                    main,
                    Form::new(GlobalMessage::Layout(Message::ExitPopUp), GlobalMessage::Layout(Message::ConfirmPopUp))
                        .push({
                            let last = remaining.last_mut().unwrap();

                            iced::TextInput::new(
                                &mut last.state,
                                "Cell name",
                                &last.value,
                                |s| GlobalMessage::Layout(Message::UpdatedName(s)),
                            ).padding(PADDING)
                        }),
                ).view(pop_up),

            State::ProvideExtrude { pop_up, name, wrap } =>
                PopUp::new(
                    main,
                    Form::new(GlobalMessage::Layout(Message::ExitPopUp), GlobalMessage::Layout(Message::ConfirmPopUp))
                        .push(
                            iced::TextInput::new(
                                &mut name.state,
                                "Group name",
                                &name.value,
                                |s| GlobalMessage::Layout(Message::UpdatedName(s)),
                            ).padding(PADDING)
                        )
                        .push(
                            iced::TextInput::new(
                                &mut wrap.state,
                                "Group wrap",
                                &wrap.value,
                                |s| GlobalMessage::Layout(Message::UpdatedFirstWrap(s)),
                            ).padding(PADDING)
                        ),
                ).view(pop_up),

            State::ProvideSprout { pop_up, last_end, last_wrap, wraps, ends } =>
                PopUp::new(
                    main,
                    Form::new(GlobalMessage::Layout(Message::ExitPopUp), GlobalMessage::Layout(Message::ConfirmPopUp))
                        .push(
                            iced::TextInput::new(
                                &mut last_end.state,
                                &format!["{} sprout's name", ends[wraps.len()].1.data().inner()],
                                &last_end.value,
                                |s| GlobalMessage::Layout(Message::UpdatedName(s)),
                            ).padding(PADDING)
                        )
                        .push(
                            iced::TextInput::new(
                                &mut last_wrap.state,
                                "Wrap's name",
                                &last_wrap.value,
                                |s| GlobalMessage::Layout(Message::UpdatedFirstWrap(s)),
                            ).padding(PADDING)
                        ),
                ).view(pop_up),

            State::ProvideSplit { pop_up, name, wrap_top, wrap_bot } =>
                PopUp::new(
                    main,
                    Form::new(GlobalMessage::Layout(Message::ExitPopUp), GlobalMessage::Layout(Message::ConfirmPopUp))
                        .push(
                            iced::TextInput::new(
                                &mut name.state,
                                "Group name",
                                &name.value,
                                |s| GlobalMessage::Layout(Message::UpdatedName(s)),
                            ).padding(PADDING)
                        )
                        .push(
                            iced::TextInput::new(
                                &mut wrap_top.state,
                                "Top part wrap",
                                &wrap_top.value,
                                |s| GlobalMessage::Layout(Message::UpdatedFirstWrap(s)),
                            ).padding(PADDING)
                        )
                        .push(
                            iced::TextInput::new(
                                &mut wrap_bot.state,
                                "Bottom part wrap",
                                &wrap_bot.value,
                                |s| GlobalMessage::Layout(Message::UpdatedSecondWrap(s)),
                            ).padding(PADDING)
                        ),
                ).view(pop_up),

            State::ProvidePass { pop_up, groups_left, last, .. } =>
                PopUp::new(
                    main,
                    Form::new(GlobalMessage::Layout(Message::ExitPopUp), GlobalMessage::Layout(Message::ConfirmPopUp))
                        .push(
                            iced::TextInput::new(
                                &mut last.state,
                                &format!["{} wrap's name", groups_left.last().unwrap().1.data().inner()],
                                &last.value,
                                |s| GlobalMessage::Layout(Message::UpdatedFirstWrap(s)),
                            ).padding(PADDING)
                        ),
                ).view(pop_up),
        }
    }
}
