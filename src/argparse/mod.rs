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
    program_name: String,
    args: HashMap<String, Arg>,
    current: Option<String>,
    non_bound: Vec<String>, // non-bound arguments
}

impl ArgParse {
    pub fn new(program_name: &str) -> Self {
        Self {
            program_name: program_name.to_string(),
            args: HashMap::new(),
            current: None,
            non_bound: Vec::new(),
        }
    }

    // Gets all non-bound arguments
    pub fn get_non_bound(&self) -> &Vec<String> {
        &self.non_bound
    }

    // Retrieves the value of the argument key
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

    // Prints help text
    pub fn print_help(&self) {
        println!("{}", self.program_name);
        println!("\narguments:");

        // Sort the arguments by name
        let mut args: Vec<&String> = self.args.keys().collect();
        args.sort();

        for key in args {
            let arg = self.args.get(key).unwrap();
            // Formats and prints the argument with indentation padding
            let mut arg_str = String::from("  ");
            arg_str.push_str(&arg.name);
            if arg.alternative.is_some() {
                arg_str.push_str(" (--");
                arg_str.push_str(arg.alternative.as_ref().unwrap());
                arg_str.push(')');
            }
            if arg.required {
                arg_str.push_str(" (required)");
            }
            if arg.default.is_some() {
                arg_str.push_str(" (default: ");
                arg_str.push_str(arg.default.as_ref().unwrap());
                arg_str.push(')');
            }
            if arg.help.is_some() {
                arg_str.push_str("\n    ");
                arg_str.push_str(arg.help.as_ref().unwrap());
            }
            // Print the argument text
            println!("{}", arg_str);
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

        true
    }

    #[cfg(not(tarpaulin_include))]
    // Parses the arguments and returns true if successful
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
    // Sets help message for the argument
    pub fn help(&mut self, help: &str) -> &mut Self {
        if let Some(last) = &self.current {
            self.args.get_mut(last).unwrap().help = Some(help.to_string());
        } else {
            panic!("No argument specified before 'help'");
        }

        self
    }

    #[allow(dead_code)]
    // Sets default value for the argument
    pub fn default(&mut self, value: &str) -> &mut Self {
        if let Some(last) = &self.current {
            self.args.get_mut(last).unwrap().default = Some(value.to_string());
        } else {
            panic!("No argument specified before 'default'");
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_argparser() {
        let mut parser = ArgParse::new("test");
        parser.arg("--help").help("Print this help message");
        parser.arg("--default").default("default");
        parser.arg("--value");

        let args = vec![
            String::from("--help"),
            String::from("--value"),
            String::from("value"),
            String::from("non-bound1"),
            String::from("non-bound2"),
        ];
        parser.internal_parse(args);

        assert_eq!(parser.get("--help"), Some("".to_string()));
        assert_eq!(parser.get("--value"), Some("value".to_string()));
        assert_eq!(parser.get("--default"), Some("default".to_string()));
        assert_eq!(parser.get("blah"), None);
        assert_eq!(parser.get_non_bound().len(), 2);
        assert_eq!(parser.get_non_bound()[0], "non-bound1".to_string());
        assert_eq!(parser.get_non_bound()[1], "non-bound2".to_string());
    }

    #[test]
    #[should_panic]
    fn test_invalid_help() {
        let mut parser = ArgParse::new("test");
        parser.help("Print this help message");
    }

    #[test]
    #[should_panic]
    fn test_invalid_default() {
        let mut parser = ArgParse::new("test");
        parser.default("default");
    }
}
