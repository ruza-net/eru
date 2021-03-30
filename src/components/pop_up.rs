use std::time::Duration;



#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct State {
    animated: f32,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Location {
    Top,
    Bottom,

    Left,
    Right,
}
impl Default for Location {
    fn default() -> Self {
        Self::Top
    }
}

pub struct PopUp<'s, Msg> {
    pub anim_duration: Duration,
    pub location: Location,

    pub pop_up: iced::Element<'s, Msg>,
    pub main_content: iced::Element<'s, Msg>,
}



impl<'s, Msg: 's> PopUp<'s, Msg> {
    pub fn new(main_content: impl Into<iced::Element<'s, Msg>>, pop_up: impl Into<iced::Element<'s, Msg>>) -> Self {
        Self {
            anim_duration: Duration::from_secs(1),
            location: fill![],

            pop_up: pop_up.into(),
            main_content: main_content.into(),
        }
    }

    accessors! {
        anim_duration: Duration,
        location: Location,

        pop_up: iced::Element<'s, Msg>,
        main_content: iced::Element<'s, Msg>,
    }

    pub fn view(self, state: &'s mut State) -> iced::Element<'s, Msg> {
        let children =
        match self.location {
            Location::Top | Location::Left => vec![self.pop_up, self.main_content],

            Location::Bottom | Location::Right => vec![self.main_content, self.pop_up],
        };

        match self.location {
            Location::Top | Location::Bottom =>
                iced::Column::with_children(children)
                .height(iced::Length::FillPortion(1))
                .into(),

            Location::Left | Location::Right =>
                iced::Row::with_children(children)
                .height(iced::Length::FillPortion(1))
                .into(),
        }
    }
}
