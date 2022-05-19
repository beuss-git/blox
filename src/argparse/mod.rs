use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug)]
pub struct Arg {
    name: String,
    alternative: Option<String>,
    value: Option<String>,
    required: bool,
    default: Option<String>,
    help: Option<String>,
    found: bool,
}

pub struct ArgParse {
    args: HashMap<String, Arg>,
    current: Option<String>,
    non_bound: Vec<String>, // non-bound arguments
}

impl ArgParse {
    pub fn new() -> Self {
        Self {
            args: HashMap::new(),
            current: None,
            non_bound: Vec::new(),
        }
    }

    pub fn get_non_bound(&self) -> &Vec<String> {
        &self.non_bound
    }

    pub fn get(&self, key: &str) -> Option<String> {
        if let Some(arg) = self.args.get(key) {
            // Only return if found
            if arg.found {
                if arg.value.is_some() {
                    return arg.value.clone();
                } else {
                    // Return *something* indicating that it was found
                    // This is a bit hacky, but it works for my little project :^)
                    return Some(String::from(""));
                }
            } else if arg.default.is_some() {
                // Return the default if it has a value
                return arg.default.clone();
            }
        }
        None
    }

    fn print_help(&self) {
        for arg in self.args.values() {
            println!("{:?}", arg);
        }
    }
    // Split for testing
    fn internal_parse(&mut self, arguments: Vec<String>) -> bool {
        // Skip the program name

        let mut i = 0;
        while i < arguments.len() {
            let name = arguments[i].clone();

            if !name.starts_with('-') {
                // We are either 'out of sync' (which I don't handle) or this is a non-bound argument
                self.non_bound.push(name.clone());
            }
            match self.args.get_mut(&name) {
                Some(arg) => {
                    // It was found
                    arg.found = true;
                    if i + 1 < arguments.len() {
                        // There is a value
                        let next = arguments[i + 1].clone();
                        if !next.starts_with('-') {
                            arg.value = Some(next);

                            // Skip the value
                            i += 1
                        }
                    }
                }
                None => {}
            }
            i += 1;
        }

        if self.args.contains_key("help") {
            self.print_help();
        }
        true
    }

    pub fn parse(&mut self) -> bool {
        let mut args = std::env::args();
        // Skip the program name
        args.next();

        self.internal_parse(args.collect::<Vec<String>>())
    }

    /// Adds an argument to the parser.
    pub fn arg(&mut self, name: &str) -> &mut Self {
        self.args.insert(
            name.to_string(),
            Arg {
                name: name.to_string(),
                alternative: None,
                value: None,
                required: false,
                default: None,
                help: None,
                found: false,
            },
        );
        // Set the current arg to this one
        self.current = Some(name.to_string());

        self
    }

    #[allow(dead_code)]
    pub fn help(&mut self, help: &str) -> &mut Self {
        if let Some(last) = &self.current {
            self.args.get_mut(last).unwrap().help = Some(help.to_string());
        } else {
            panic!("No argument specified before 'help'");
        }

        self
    }

    #[allow(dead_code)]
    pub fn default(&mut self, value: String) -> &mut Self {
        if let Some(last) = &self.current {
            self.args.get_mut(last).unwrap().default = Some(value);
        } else {
            panic!("No argument specified before 'default'");
        }

        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_argparser() {
        use super::*;

        let mut parser = ArgParse::new();
        parser.arg("--help").help("Print this help message");
        parser.arg("--default").default(String::from("default"));
        parser.arg("--value");

        let args = vec![
            String::from("--help"),
            String::from("--value"),
            String::from("value"),
        ];
        parser.internal_parse(args);

        assert_eq!(parser.get("--help"), Some("".to_string()));
        assert_eq!(parser.get("--value"), Some("value".to_string()));
        assert_eq!(parser.get("--default"), Some("default".to_string()));
        assert_eq!(parser.get("blah"), None);
    }
}
