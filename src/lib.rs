use std::{collections::HashMap, default, usize};

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

    fn next_while(&mut self, predicate: fn(char) -> bool) {
        while self.is_valid() && predicate(self.peek().unwrap()) {
            self.next();
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

#[derive(Debug)]
enum JsonType {
    Obj(HashMap<String, JsonType>),
    Arr(Vec<JsonType>),
    Str(String),
    Boolean(bool),
    Number(i32),
    Null,
}

fn parse_json_null(cursor: &mut CharsCursor) -> Result<JsonType, Box<dyn std::error::Error>> {
    let mut null_str = String::default();
    while cursor.is_valid() {
        let ch = cursor.peek().unwrap();

        if !ch.is_alphabetic() {
            break;
        }

        null_str.push(ch);
        cursor.next();
    }

    if null_str.as_str() != "null" {
        return Err(format!("illegal value {}", null_str.as_str()).into());
    }

    Ok(JsonType::Null)
}

fn parse_json_str(cursor: &mut CharsCursor) -> Result<JsonType, Box<dyn std::error::Error>> {
    let mut result = String::default();
    cursor.next();

    while cursor.is_valid() {
        let char = cursor.peek().unwrap();

        match char {
            '\"' => {
                break;
            }
            other => {
                result.push(other);
                cursor.next();
            }
        }
    }
    if !cursor.is_valid() {
        return Err("expected '\"'".into());
    }

    cursor.next();

    Ok(JsonType::Str(result))
}

fn parse_json_boolean(cursor: &mut CharsCursor) -> Result<JsonType, Box<dyn std::error::Error>> {
    let mut boolean_str = String::default();
    while cursor.is_valid() {
        let ch = cursor.peek().unwrap();
        if !ch.is_alphabetic() {
            break;
        }
        boolean_str.push(ch);
        cursor.next();
    }

    let result = match boolean_str.as_str() {
        "true" => true,
        "false" => false,
        other => {
            return Err(format!("illeagl value {}", other).into());
        }
    };

    Ok(JsonType::Boolean(result))
}

fn parse_json_number(cursor: &mut CharsCursor) -> Result<JsonType, Box<dyn std::error::Error>> {
    let mut number_str = String::default();

    while cursor.is_valid() {
        let ch = cursor.peek().unwrap();
        if !ch.is_digit(10) {
            break;
        }
        number_str.push(ch);
        cursor.next();
    }

    let result = number_str.parse::<i32>()?;
    Ok(JsonType::Number(result))
}

fn parse_json_key_value(
    cursor: &mut CharsCursor,
) -> Result<(String, JsonType), Box<dyn std::error::Error>> {
    let mut key = String::default();

    cursor.next();
    while cursor.is_valid() {
        let char = cursor.peek().unwrap();
        match char {
            '\"' => {
                break;
            }
            other => {
                key.push(other);
                cursor.next();
            }
        }
    }
    if !cursor.is_valid() {
        return Err("expected '\"'".into());
    }

    cursor.next();
    cursor.next_while(|ch| ch == ' ' || ch == '\t');
    if !cursor.is_valid() {
        return Err("expected ':'".into());
    } else if cursor.peek().unwrap() != ':' {
        return Err(format!("error char {}", cursor.peek().unwrap()).into());
    }
    cursor.next();

    cursor.next_while(|ch| ch == ' ' || ch == '\t');

    if !cursor.is_valid() {
        return Err("missing value".into());
    }

    let value = parse_json_value(cursor)?;

    Ok((key, value))
}

fn parse_json_arr(cursor: &mut CharsCursor) -> Result<JsonType, Box<dyn std::error::Error>> {
    cursor.next();

    let mut json_arr: Vec<JsonType> = Vec::default();

    while cursor.is_valid() {
        let value = parse_json_value(cursor)?;
        json_arr.push(value);

        cursor.next_while(|ch| ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r');

        if !cursor.is_valid() {
            return Err("expected ]".into());
        }

        let ch = cursor.peek().unwrap();
        if ch == ',' {
            cursor.next();
        } else {
            break;
        }
    }

    cursor.next_while(|ch| ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r');

    if !cursor.is_valid() {
        return Err("expected ]".into());
    }

    if cursor.peek().unwrap() != ']' {
        return Err(format!("error char {}", cursor.peek().unwrap()).into());
    }

    cursor.next();

    Ok(JsonType::Arr(json_arr))
}

fn parse_json_obj(cursor: &mut CharsCursor) -> Result<JsonType, Box<dyn std::error::Error>> {
    let mut result = HashMap::<String, JsonType>::default();
    cursor.next();

    while cursor.is_valid() {
        cursor.next_while(|ch| ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n');
        if !cursor.is_valid() {
            return Err("expected '}'".into());
        } else if cursor.peek().unwrap() != '\"' {
            return Err(format!("error char {},expected '\"'", cursor.peek().unwrap()).into());
        }
        let (key, value) = parse_json_key_value(cursor)?;
        result.insert(key, value);
        cursor.next_while(|ch| ch == ' ' || ch == '\t');

        if !cursor.is_valid() {
            return Err("expected '}'".into());
        }

        let ch = cursor.peek().unwrap();
        if ch != ',' {
            if ch == '\r' || ch == '\n' || ch == '}' {
                break;
            } else {
                return Err(format!("error char {}", ch).into());
            }
        } else {
            cursor.next();
        }
    }

    cursor.next_while(|ch| ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r');
    if !cursor.is_valid() {
        return Err("expected '}'".into());
    } else if cursor.peek().unwrap() != '}' {
        return Err(format!("error char {}", cursor.peek().unwrap()).into());
    }

    cursor.next();

    Ok(JsonType::Obj(result))
}

fn parse_json_value(cursor: &mut CharsCursor) -> Result<JsonType, Box<dyn std::error::Error>> {
    cursor.next_while(|ch| ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n');

    if !cursor.is_valid() {
        return Err("missing value".into());
    }

    let beg_mark = cursor.peek().unwrap();

    let result = match beg_mark {
        '[' => parse_json_number(cursor)?,
        '\"' => parse_json_str(cursor)?,
        '0'..='9' => parse_json_number(cursor)?,
        't' | 'f' => parse_json_boolean(cursor)?,
        '{' => parse_json_obj(cursor)?,
        'n' => parse_json_null(cursor)?,
        other => {
            return Err(format!("error char {}", other).into());
        }
    };

    Ok(result)
}

#[cfg(test)]
mod test {
    use std::{fs::read_to_string, process::exit};

    use crate::{
        parse_json_arr, parse_json_boolean, parse_json_null, parse_json_number, parse_json_obj,
        parse_json_str, CharsCursor,
    };

    #[test]
    fn test_parse_obj() {
        let json_str = read_to_string("case1.json").unwrap();
        let chars: Vec<_> = json_str.chars().collect();
        let mut cursor = CharsCursor::new(&chars);

        let result = parse_json_obj(&mut cursor).unwrap_or_else(|err| {
            eprint!("error:{}", err);
            exit(1);
        });

        eprintln!("result:{:#?}", result);
    }

    #[test]
    fn test_parse_arr() {
        let chars: Vec<_> = "[1, null, false, 4,523,{\"name\" : \"hcc\"} ]"
            .chars()
            .collect();
        let mut cursor = CharsCursor::new(&chars);

        let result = parse_json_arr(&mut cursor).unwrap();

        eprintln!("result:{:#?}", result);
    }

    #[test]
    fn test_parse_null() {
        let chars: Vec<_> = "null".chars().collect();
        let mut cursor = CharsCursor::new(&chars);

        let result = parse_json_null(&mut cursor).unwrap();

        eprintln!("result:{:?}", result);
    }

    #[test]
    fn test_parse_number() {
        let chars: Vec<_> = "12324".chars().collect();
        let mut cursor = CharsCursor::new(&chars);

        let result = parse_json_number(&mut cursor).unwrap();

        eprintln!("result:{:?}", result);
    }

    #[test]
    fn test_parse_boolean() {
        let chars: Vec<_> = "true".chars().collect();
        let mut cursor = CharsCursor::new(&chars);

        let result = parse_json_boolean(&mut cursor).unwrap();

        eprintln!("result:{:?}", result);
    }

    #[test]
    fn test_parse_str() {
        let chars: Vec<_> = "\"hcc\"".chars().collect();
        let mut cursor = CharsCursor::new(&chars);

        let result = parse_json_str(&mut cursor).unwrap();

        eprintln!("result:{:?}", result);
    }

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
