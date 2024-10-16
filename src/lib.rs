use std::{collections::HashMap, usize};

enum CursorState {
    Valid(usize),
    Lowwer,
    Upper,
}

struct CharsCursor<'a> {
    chars: &'a [char],
    cur: CursorState,
}

impl<'a> CharsCursor<'a> {
    fn new(chars: &'a [char]) -> CharsCursor {
        CharsCursor {
            chars,
            cur: CursorState::Valid(0),
        }
    }

    fn is_valid(&self) -> bool {
        if let CursorState::Valid(_) = self.cur {
            true
        } else {
            false
        }
    }

    fn prev(&mut self) {
        if let CursorState::Valid(x) = self.cur {
            if x != 0 {
                self.cur = CursorState::Valid(x - 1)
            } else {
                self.cur = CursorState::Lowwer
            }
        }
    }

    fn next(&mut self) {
        if let CursorState::Valid(x) = self.cur {
            if x != self.chars.len() - 1 {
                self.cur = CursorState::Valid(x + 1)
            } else {
                self.cur = CursorState::Upper
            }
        }
    }

    fn peek(&self) -> Option<char> {
        if let CursorState::Valid(x) = self.cur {
            Some(self.chars[x])
        } else {
            None
        }
    }
}

enum JsonType {
    Obj(HashMap<String, JsonType>),
    Str(String),
}

fn parse_json_obj(chars: &[char]) {
    let mut cur = 0;
    while cur < chars.len() {}
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use crate::CharsCursor;

    #[test]
    fn test_peeker() {
        let json_str = read_to_string("case1.json").unwrap();
        let chars: Vec<_> = json_str.chars().collect();
        let mut cursor = CharsCursor::new(&chars);

        while cursor.is_valid() {
            let char = cursor.peek().unwrap();
            cursor.next();
            eprintln!("{}", char);
        }
    }
}
