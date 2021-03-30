use std::ops;

use iced::button;

use crate::model::Render;
use crate::behavior::{ Clickable, SimpleView };

use super::viewing::{ Message, ViewIndex };



#[derive(Debug, Clone)]
pub struct Selectable<Data> {
    val: Data,

    selected: bool,

    select: button::State,
}



const HEIGHT: u16 = 8;
use crate::styles::container::{ PADDING, cell::SPACING };


impl<Data> Selectable<Data> {
    pub const fn inner(&self) -> &Data {
        &self.val
    }

    pub fn select(&mut self) {
        self.selected = !self.selected;
    }

    pub fn unselect(&mut self) {
        self.selected = false;
    }

    pub const fn selected(&self) -> bool {
        self.selected
    }
}

impl<Data: SimpleView> Selectable<Data> {
    pub fn view_cell<'s>(
        &'s mut self,
        index: ViewIndex,
        content_width: u16,
        contents: Option<iced::Element<'s, Message>>,
        render: Render,
    ) -> ((u16, u16), iced::Element<'s, Message>) {

        let (width, data) = self.view_data();

        let width = width.max(content_width);

        let contents =
            if let Some(contents) = contents {
                iced::Column::new()
                    .push(contents)
                    .push(data)
                    .align_items(iced::Align::Center)
                    .width(width.into())
                    .into()

            } else {
                data
            };

        let style = if self.selected {
            crate::styles::container::SELECTED_CELL

        } else {
            crate::styles::container::CELL
        };

        let mut cell =
        iced::Button::new(self.state(), contents)
            .style(style)
            .width(width.into())
            .padding(0);

        if render == Render::Interactive {
            cell = cell.on_press(Message::Select(index));
        }

        ((HEIGHT + 2 * SPACING, width), cell.into())
    }

    pub fn view_data(&self) -> (u16, iced::Element<'static, Message>) {
        let val = self.val.view().1;

        let width = self.width();

        (
            width,
            iced::Element::from(
                iced::Container::new(val)
                .align_x(iced::Align::Center)
                .width(width.into())
                .padding(PADDING)
            ).map(|_| Message::Idle),
        )
    }

    pub fn width(&self) -> u16 {
        let width = self.val.view().0;

        width + 2 * PADDING
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
    fn view(&self) -> (u16, iced::Element<'static, ()>) {
        self.val.view()
    }
}
impl<Data> Clickable for Selectable<Data> {
    fn state(&mut self) -> &mut button::State {
        &mut self.select
    }
}

impl<Data> ops::Deref for Selectable<Data> {
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}
