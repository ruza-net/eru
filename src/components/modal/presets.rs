#[derive(Debug, Clone, Default)]
pub struct Confirm {
    ok_state: iced::button::State,
}


impl super::ModalInteract for Confirm {
    type Message = ();

    fn interactions(&mut self) -> Vec<iced::Element<Self::Message>> {
        vec![
            iced::Button::new(&mut self.ok_state, iced::Text::new("Ok"))
                .on_press(())
                .into()
        ]
    }
}
