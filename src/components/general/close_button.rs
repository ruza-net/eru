pub struct CloseButton<'s, Msg> {
    state: &'s mut iced::button::State,
    pub is_arrow: bool,
    pub on_press: Option<Msg>,
}


impl<'s, Msg> CloseButton<'s, Msg> {
    pub fn new(state: &'s mut iced::button::State, is_arrow: bool) -> Self {
        Self {
            state,
            is_arrow,
            on_press: None,
        }
    }

    pub fn on_press(mut self, on_press: Msg) -> Self {
        self.on_press = Some(on_press);
        self
    }

    pub fn is_arrow(mut self, is_arrow: bool) -> Self {
        self.is_arrow = is_arrow;
        self
    }
}

impl<'s, Msg: Clone> CloseButton<'s, Msg> {
    pub fn into_button(self) -> iced::Button<'s, Msg> {
        let mut btn = iced::Button::new(
            self.state,
            iced::Text::new(
                if self.is_arrow {
                    ">"
                } else {
                    "X"
                }
            )
        );

        if let Some(on_press) = self.on_press {
            btn = btn.on_press(on_press);
        }

        btn
    }
}

impl<'s, Msg: Clone> From<CloseButton<'s, Msg>> for iced::Button<'s, Msg> {
    fn from(this: CloseButton<'s, Msg>) -> Self {
        this.into_button().into()
    }
}
