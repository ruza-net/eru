#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Render {
    Interactive,
    InteractiveNoLabel,

    Static,
}
