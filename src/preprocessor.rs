use std::collections::HashMap;

trait State {
    fn enter_state();
    fn exit_state();
}

enum States {
    Normal,
    Macro{ name: String }
}

pub fn process(text: &str, macros_map: &HashMap<String, Box<impl Fn() -> String>>)  -> String {
    let mut state = States::Normal;
    let mut new_text = "".to_string();

    for c in text.chars() {
        match c {
            '$' => state = States::Macro {name : "".to_string()},
            _ => match state {
                States::Normal => new_text.push(c),
                States::Macro{ref mut name} => {
                    if c.is_alphanumeric() {
                        name.push(c);
                    } else {
                        match macros_map.get(name) {
                            Some(val) => new_text.push_str(&val()),
                            None => ()
                        };
                        state = States::Normal;
                        new_text.push(c);
                    }
                }
            }
        };
    }

    new_text
}
