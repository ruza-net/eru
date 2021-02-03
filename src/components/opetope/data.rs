use iced::button;

use crate::behavior::{ Clickable, SimpleView };



#[derive(Debug, Clone)]
pub struct Selectable<Data> {
    val: Data,
    select: button::State,
}



impl<Data> From<Data> for Selectable<Data> {
    fn from(val: Data) -> Self {
        Self {
            val,
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
