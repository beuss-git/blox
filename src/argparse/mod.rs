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
}

impl ArgParse {
    pub fn new() -> Self {
        Self {
            args: HashMap::new(),
            current: None,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        if let Some(arg) = self.args.get(key) {
            // Only return if found
            if arg.found {
                if arg.value.is_some() {
                    return arg.value.clone();
                } else if arg.default.is_some() {
                    // Return the default if it has a value
                    return arg.default.clone();
                } else {
                    // Return *something* indicating that it was found
                    // This is a bit hacky, but it works for my little project :^)
                    return Some(String::from(""));
                }
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

        for mut i in 0..arguments.len() {
            let name = arguments[i].clone();
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

    pub fn help(&mut self, help: &str) -> &mut Self {
        if let Some(last) = &self.current {
            self.args.get_mut(last).unwrap().help = Some(help.to_string());
        } else {
            panic!("No argument specified before 'help'");
        }

        self
    }

    pub fn default(&mut self, value: String) -> &mut Self {
        if let Some(last) = &self.current {
            self.args.get_mut(last).unwrap().default = Some(value);
        } else {
            panic!("No argument specified before 'default'");
        }

        self
    }
}
