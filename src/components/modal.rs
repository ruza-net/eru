pub mod presets;

use iced_aw::{ modal, Card };



pub trait ModalInteract {
    type Message;

    fn interactions(&mut self) -> Vec<iced::Element<Self::Message>>;
}


#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Kind {
    Info,
    Error,
}


pub struct Modal<S> {
    state: modal::State<S>,

    kind: Kind,

    title: String,
    body: String,
}



impl<S: Default> Modal<S> {
    pub fn new(title: impl ToString, body: impl ToString, kind: Kind) -> Self {
        let mut state = modal::State::new(fill![]);
        state.show(true);

        Self {
            state,

            kind,

            title: title.to_string(),
            body: body.to_string(),
        }
    }
}

impl<S: ModalInteract> Modal<S>
where S::Message: 'static + Clone
{
    pub fn view<'m, Msg: 'static + Clone>(
        &'m mut self,
        content: impl Into<iced::Element<'m, Msg>>,
        msg: impl Fn(S::Message) -> Msg + Clone + 'static,
    ) -> iced::Element<'m, Msg>
    {
        let title = &self.title;
        let body = &self.body;

        modal::Modal::new(&mut self.state, content, move |state| {
            iced::Container::new(
                Card::new(
                    iced::Text::new(title),
                    iced::Text::new(body),
                )
                .foot(
                    iced::Row::with_children(
                        state
                            .interactions()
                            .into_iter()
                            .map(|el| el.map(msg.clone()))
                            .collect()
                    )
                )
            )
            .padding(crate::styles::container::PADDING)
            .into()
        })
        .into()
    }
}
