pub trait SimpleView {
    fn view(&self) -> iced::Element<'static, ()>;
}


pub trait Clickable {
    fn state(&mut self) -> &mut iced::button::State;
}



impl SimpleView for String {
    fn view(&self) -> iced::Element<'static, ()> {
        iced::Text::new(self.to_string()).into()
    }
}
