pub trait View {
    fn view(&self) -> iced::Element<'static, ()>;
}
