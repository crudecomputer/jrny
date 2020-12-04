pub enum Action {
    Ignore,
    Append,
    Carry,
}

pub trait State {
    fn can_terminate(&self) -> bool {
        false
    }

    fn accept(&self, grapheme: &str) -> (Action, Box<dyn State>);
}

pub struct Start;
pub struct InString;
pub struct InDelimitedIdentifier;
pub struct MightStartInlineComment;
pub struct InInlineComment;
pub struct MightStartBlockComment;
pub struct InBlockComment;
pub struct MightEndBlockComment;

impl State for Start {
    fn can_terminate(&self) -> bool {
        true
    }

    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "'" => (Action::Append, Box::new(InString)),
            "\"" => (Action::Append, Box::new(InDelimitedIdentifier)),
            "-" => (Action::Carry, Box::new(MightStartInlineComment)),
            "/" => (Action::Carry, Box::new(MightStartBlockComment)),
            _ => (Action::Append, Box::new(Start)),
        }
    }
}

impl State for InString {
    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "'" => (Action::Append, Box::new(Start)),
            _ => (Action::Append, Box::new(InString)),
        }
    }
}

impl State for InDelimitedIdentifier {
    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "\"" => (Action::Append, Box::new(Start)),
            _ => (Action::Append, Box::new(InDelimitedIdentifier)),
        }
    }
}

impl State for MightStartInlineComment {
    fn can_terminate(&self) -> bool {
        true
    }

    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "'" => (Action::Append, Box::new(InString)),
            "\"" => (Action::Append, Box::new(InDelimitedIdentifier)),
            "--" => (Action::Ignore, Box::new(InInlineComment)),
            "/" => (Action::Carry, Box::new(MightStartBlockComment)),
            _ => (Action::Append, Box::new(Start)),
        }
    }
}

impl State for InInlineComment {
    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "\n" => (Action::Append, Box::new(Start)),
            _ => (Action::Ignore, Box::new(InInlineComment)),
        }
    }
}

impl State for MightStartBlockComment {
    fn can_terminate(&self) -> bool {
        true
    }

    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "'" => (Action::Append, Box::new(InString)),
            "\"" => (Action::Append, Box::new(InDelimitedIdentifier)),
            "-" => (Action::Ignore, Box::new(MightStartInlineComment)),
            "/*" => (Action::Carry, Box::new(InBlockComment)),
            _ => (Action::Append, Box::new(Start)),
        }
    }
}

impl State for InBlockComment {
    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "*" => (Action::Carry, Box::new(MightEndBlockComment)),
            _ => (Action::Ignore, Box::new(InBlockComment)),
        }
    }
}

impl State for MightEndBlockComment {
    fn accept(&self, s: &str) -> (Action, Box<dyn State>) {
        match s {
            "*/" => (Action::Ignore, Box::new(Start)),
            _ => (Action::Ignore, Box::new(InBlockComment)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_can_terminate() {
        assert_eq!(Start.can_terminate(), true);
        assert_eq!(InString.can_terminate(), false); // Inside string
        assert_eq!(InDelimitedIdentifier.can_terminate(), false); // Inside quoted
        assert_eq!(MightStartInlineComment.can_terminate(), true);
        assert_eq!(InInlineComment.can_terminate(), false); // Inside comment
        assert_eq!(MightStartBlockComment.can_terminate(), true);
        assert_eq!(InBlockComment.can_terminate(), false); // Inside comment
        assert_eq!(MightEndBlockComment.can_terminate(), false); // Inside comment
    }
}
