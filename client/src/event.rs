use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum State {
    Connecting,
    Connected,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub enum SessionEvent {
    Connected,    // state: Connecting -> Connected,
    Disconnected, // state: Connected -> Connecting,
    AuthFailed,   // state: Connecting -> AuthFailed,
}

#[derive(Clone, Debug)]
pub enum Type {
    Session,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Event {
    pub r#type: Type,
    pub state: State,
    pub path: String,
}
