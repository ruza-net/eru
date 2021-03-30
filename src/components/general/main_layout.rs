use crate::components::{
    opetope,

    sidebar::Sidebar,

    app::{ GlobalMessage, Data },
    pop_up::{ self, PopUp },

    general::close_button,
};


#[derive(Debug, Clone)]
pub enum Message {
    UpdatedName(String),
    UpdatedFirstWrap(String),
    UpdatedSecondWrap(String),

    ClosePopUp,
}

const ERR_DURATION: u64 = 5;



#[derive(Default, Debug, Clone)]
pub struct NameSlot {
    pub state: iced::text_input::State,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum State {
    Default,

    ProvideExtrude {
        pop_up: pop_up::State,
        close: iced::button::State,

        name: NameSlot,
        wrap: NameSlot,
    },

    ProvideSprout {
        pop_up: pop_up::State,
        close: iced::button::State,

        last_end: NameSlot,
        last_wrap: NameSlot,
        
        wraps: Vec<Data>,
        ends: Vec<(opetope::ViewIndex, opetope::Cell<Data>, Option<Data>)>,
    },

    ProvideSplit {
        pop_up: pop_up::State,
        close: iced::button::State,

        name: NameSlot,
        wrap_top: NameSlot,
        wrap_bot: NameSlot,
    },

    ProvidePass {
        pop_up: pop_up::State,
        close: iced::button::State,

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
        Self::ProvideExtrude { pop_up: fill![], close: fill![], name: fill![], wrap: fill![] }
    }

    pub fn sprout() -> Self {
        Self::ProvideSprout { pop_up: fill![], close: fill![], last_end: fill![], last_wrap: fill![], ends: fill![], wraps: fill![] }
    }

    pub fn split() -> Self {
        Self::ProvideSplit { pop_up: fill![], close: fill![], name: fill![], wrap_top: fill![], wrap_bot: fill![] }
    }

    pub fn pass(groups_left: Vec<(opetope::Face, opetope::MetaCell<Data>)>) -> Self {
        Self::ProvidePass { pop_up: fill![], close: fill![], last: fill![], wraps: fill![], groups_left }
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

    error: Option<opetope::Error>,

    pub state: State,
    sidebar: Sidebar,
}



impl Layout {
    pub fn error(&mut self, e: opetope::Error) {
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

        let mut main =
        iced::Row::new()
            .push(sidebar)
            .push(opetope)
            .into();

        if self.error_countdown > 0 {
            let err_text = self.error.as_ref().unwrap().to_string();

            let err_msg = iced::Container::new(iced::Text::new(err_text))
                .width(iced::Length::Fill)
                .style(crate::styles::container::Error);

            main =
            PopUp::new(main, err_msg)
                .location(pop_up::Location::Top)
                .view(&mut self.error_pop_up)
        }

        match &mut self.state {
            State::Default =>
                main,

            State::ProvideExtrude { pop_up, close, name, wrap } =>
                PopUp::new(
                    main,
                    iced::Container::new(
                        iced::Row::new()
                        .push(
                            iced::TextInput::new(
                                &mut name.state,
                                "Group name",
                                &name.value,
                                |s| GlobalMessage::Layout(Message::UpdatedName(s)),
                            )
                        )
                        .push(
                            iced::TextInput::new(
                                &mut wrap.state,
                                "Group wrap",
                                &wrap.value,
                                |s| GlobalMessage::Layout(Message::UpdatedFirstWrap(s)),
                            )
                        )
                        .push(iced::Space::with_width(iced::Length::Fill))
                        .push(
                            close_button::CloseButton::new(close, false)
                            .on_press(GlobalMessage::Layout(Message::ClosePopUp))
                            .into_button()
                        )
                    )
                    .style(crate::styles::container::PopUp),
                ).view(pop_up),

            State::ProvideSprout { pop_up, close, last_end, last_wrap, wraps, ends } =>
                PopUp::new(
                    main,
                    iced::Container::new(
                        iced::Row::new()
                        .push(
                            iced::TextInput::new(
                                &mut last_end.state,
                                &format!["{} sprout's name", ends[wraps.len()].1.data().inner()],
                                &last_end.value,
                                |s| GlobalMessage::Layout(Message::UpdatedName(s)),
                            )
                        )
                        .push(
                            iced::TextInput::new(
                                &mut last_wrap.state,
                                "Wrap's name",
                                &last_wrap.value,
                                |s| GlobalMessage::Layout(Message::UpdatedFirstWrap(s)),
                            )
                        )
                        .push(iced::Space::with_width(iced::Length::Fill))
                        .push(
                            close_button::CloseButton::new(close, true)
                            .on_press(GlobalMessage::Layout(Message::ClosePopUp))
                            .into_button()
                        )
                    )
                    .style(crate::styles::container::PopUp),
                ).view(pop_up),

            State::ProvideSplit { pop_up, close, name, wrap_top, wrap_bot } =>
                PopUp::new(
                    main,
                    iced::Container::new(
                        iced::Row::new()
                        .push(
                            iced::TextInput::new(
                                &mut name.state,
                                "Group name",
                                &name.value,
                                |s| GlobalMessage::Layout(Message::UpdatedName(s)),
                            )
                        )
                        .push(
                            iced::TextInput::new(
                                &mut wrap_top.state,
                                "Top part wrap",
                                &wrap_top.value,
                                |s| GlobalMessage::Layout(Message::UpdatedFirstWrap(s)),
                            )
                        )
                        .push(
                            iced::TextInput::new(
                                &mut wrap_bot.state,
                                "Bottom part wrap",
                                &wrap_bot.value,
                                |s| GlobalMessage::Layout(Message::UpdatedSecondWrap(s)),
                            )
                        )
                        .push(iced::Space::with_width(iced::Length::Fill))
                        .push(
                            close_button::CloseButton::new(close, false)
                            .on_press(GlobalMessage::Layout(Message::ClosePopUp))
                            .into_button()
                        )
                    )
                    .style(crate::styles::container::PopUp),
                ).view(pop_up),

            State::ProvidePass { pop_up, close, groups_left, last, .. } =>
                PopUp::new(
                    main,
                    iced::Container::new(
                        iced::Row::new()
                        .push(
                            iced::TextInput::new(
                                &mut last.state,
                                &format!["{} wrap's name", groups_left.last().unwrap().1.data().inner()],
                                &last.value,
                                |s| GlobalMessage::Layout(Message::UpdatedFirstWrap(s)),
                            )
                        )
                        .push(iced::Space::with_width(iced::Length::Fill))
                        .push(
                            close_button::CloseButton::new(close, true)
                            .on_press(GlobalMessage::Layout(Message::ClosePopUp))
                            .into_button()
                        )
                    )
                    .style(crate::styles::container::PopUp),
                ).view(pop_up),
        }
    }
}
