use std::time::Duration;

use crate::components::general::CloseButton;



#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct State {
    animated: f32,

    close: iced::button::State,
    confirm: iced::button::State,
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

    pub pop_up: Form<'s, Msg>,
    pub main_content: iced::Element<'s, Msg>,
}

enum Data<Msg> {
    Dialog { on_close: Msg, on_confirm: Msg },

    Error,
}
pub struct Form<'s, Msg> {
    children: Vec<iced::Element<'s, Msg>>,

    data: Data<Msg>,
}

impl<'s, Msg> Form<'s, Msg> {
    pub fn new(on_close: Msg, on_confirm: Msg) -> Self {
        Self {
            children: vec![],
            data: Data::Dialog { on_close, on_confirm },
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            children: vec![iced::Text::new(message).into()],
            data: Data::Error,
        }
    }

    pub fn push(mut self, child: impl Into<iced::Element<'s, Msg>>) -> Self {
        self.children.push(child.into());
        self
    }
}

macro_rules! view_form {
    ( $view_fn:ident => $element:ident ~ $align:ident ) => {
        pub fn $view_fn(self, states: Option<(&'s mut iced::button::State, &'s mut iced::button::State)>) -> iced::Element<'s, Msg> {
            let mut children = self.children;
            let mut is_error = true;


            if let Data::Dialog { on_close, on_confirm } = self.data {
                is_error = false;

                let (close, confirm) = states.unwrap();

                children.insert(0, CloseButton::cross().on_press(on_close).view(close));
                children.push(CloseButton::arrow().on_press(on_confirm).view(confirm));
            }

            let pop_up =
            iced::Container::new(
                iced::$element::with_children(children)
            )
            .padding(5)
            .$align(iced::Align::Center)
            .width(iced::Length::Fill);

            if is_error {
                pop_up.style(crate::styles::container::Error)

            } else {
                pop_up.style(crate::styles::container::PopUp)
            }
            .into()
        }
    };
}
impl<'s, Msg: 'static + Clone + Default> Form<'s, Msg> {
    view_form! { view_row => Row ~ align_y }
    view_form! { view_column => Column ~ align_x }
}



impl<'s, Msg> PopUp<'s, Msg> {
    pub fn new(main_content: impl Into<iced::Element<'s, Msg>>, pop_up: Form<'s, Msg>) -> Self {
        Self {
            anim_duration: Duration::from_secs(1),
            location: fill![],

            pop_up,
            main_content: main_content.into(),
        }
    }

    accessors! {
        anim_duration: Duration,
        location: Location,

        pop_up: Form<'s, Msg>,
        main_content: iced::Element<'s, Msg>,
    }
}

impl<'s, Msg: 'static + Clone + Default> PopUp<'s, Msg> {
    pub fn view(self, state: &'s mut State) -> iced::Element<'s, Msg> {
        let states = Some((&mut state.close, &mut state.confirm));

        let pop_up =
        match self.location {
            Location::Top | Location::Bottom => self.pop_up.view_row(states),

            Location::Left | Location::Right => self.pop_up.view_column(states),
        };

        let children =
        match self.location {
            Location::Top | Location::Left => vec![pop_up, self.main_content],

            Location::Bottom | Location::Right => vec![self.main_content, pop_up],
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
