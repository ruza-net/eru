pub trait SimpleView {
    fn view(&self) -> iced::Element<'static, ()>;
}


pub trait Clickable {
    fn state(&mut self) -> &mut iced::button::State;
}



impl<S: ToString> SimpleView for S {
    fn view(&self) -> iced::Element<'static, ()> {
        iced::Text::new(self.to_string()).into()
    }
}
