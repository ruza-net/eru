pub trait SimpleView {
    fn view(&self) -> (u16, iced::Element<'static, ()>);
}

pub trait View {
    type Message;

    fn view(&mut self) -> iced::Element<Self::Message>;
}


pub trait Clickable {
    fn state(&mut self) -> &mut iced::button::State;
}



impl<S: ToString> SimpleView for S {
    fn view(&self) -> (u16, iced::Element<'static, ()>) {
        let s = self.to_string();

        (21 * s.len() as u16 / 2, iced::Text::new(s).into())// TODO: account for different char widths
    }
}

impl<V: SimpleView> View for V {
    type Message = ();

    fn view(&mut self) -> iced::Element<Self::Message> {
        SimpleView::view(self).1
    }
}
