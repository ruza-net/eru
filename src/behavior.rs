pub trait View {
    type Msg;

    fn view(&self) -> iced::Element<'static, Self::Msg>;
}

pub trait SimpleView: View<Msg = ()> {}
impl<X: View<Msg = ()>> SimpleView for X {}


impl View for String {
    type Msg = ();

    fn view(&self) -> iced::Element<'static, Self::Msg> {
        iced::Text::new(self).into()
    }
}
