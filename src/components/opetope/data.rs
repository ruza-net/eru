use iced::button;

use crate::behavior::{ Clickable, SimpleView };

use super::viewing::{ Message, ViewIndex };



#[derive(Debug, Clone)]
pub struct Selectable<Data> {
    val: Data,

    selected: bool,
    select: button::State,
}



impl<Data> Selectable<Data> {
    pub fn select(&mut self) {
        self.selected = !self.selected;
    }

    pub fn unselect(&mut self) {
        self.selected = false;
    }

    pub fn selected(&self) -> bool {
        self.selected
    }
}

impl<Data: SimpleView> Selectable<Data> {
    pub fn view_cell<'s>(&'s mut self, index: ViewIndex, contents: Option<iced::Element<'s, Message>>) -> iced::Element<'s, Message> {
        let contents = if let Some(contents) = contents {
                iced::Column::new()
                    .push(contents)
                    .push(self.view_data())
                    .align_items(iced::Align::Center)
                    .into()

            } else {
                self.view_data()
            };

        let style = if self.selected {
            crate::styles::container::SELECTED_CELL

        } else {
            crate::styles::container::CELL
        };

        iced::Button::new(self.state(), contents)
            .style(style)
            .padding(crate::styles::container::PADDING)
            .on_press(Message::Select(index))
            .into()
    }

    pub fn view_data(&self) -> iced::Element<'static, Message> {
        self.val.view().map(|_| Message::Idle)
    }
}

impl<Data> From<Data> for Selectable<Data> {
    fn from(val: Data) -> Self {
        Self {
            val,

            selected: false,
            select: fill![],
        }
    }
}

impl<Data: SimpleView> SimpleView for Selectable<Data> {
    fn view(&self) -> iced::Element<'static, ()> {
        self.val.view()
    }
}
impl<Data> Clickable for Selectable<Data> {
    fn state(&mut self) -> &mut button::State {
        &mut self.select
    }
}
