use crate::model::Render;
use crate::components::general::Tooltip;



pub struct CloseButton<Msg> {
    pub is_arrow: bool,
    pub on_press: Option<Msg>,
}


impl<Msg> CloseButton<Msg> {
    pub fn cross() -> Self {
        Self {
            is_arrow: false,
            on_press: None,
        }
    }

    pub fn arrow() -> Self {
        Self {
            is_arrow: true,
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

impl<Msg: 'static + Clone + Default> CloseButton<Msg> {
    pub fn view(self, state: &mut iced::button::State) -> iced::Element<Msg> {
        let res = if self.is_arrow {
            "res/img/arrow"
        } else {
            "res/img/close"
        };

        let mut btn = Tooltip::from_file(res);

        if let Some(on_press) = self.on_press {
            btn.on_press(on_press);
        }

        btn.view_with_state(state, None, Render::Interactive)
    }
}
