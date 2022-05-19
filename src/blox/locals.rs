#[derive(Clone)]
pub struct Locals {
    stack: Vec<Local>,
    locals_count: u8,
    scope_depth: usize,
}
impl Locals {
    pub fn new() -> Self {
        Self {
            stack: vec![Local::new(); u8::MAX as usize],
            locals_count: 0,
            scope_depth: 0,
        }
    }
    pub fn scope_depth(&self) -> usize {
        self.scope_depth
    }
    pub fn is_full(&self) -> bool {
        self.locals_count == u8::MAX
    }
    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// Returns the amount of locals removed from the stack.
    pub fn end_scope(&mut self) -> usize {
        self.scope_depth -= 1;

        let previous_count = self.locals_count;

        // I would have loved to make this more functional, but I'm not sure how to do that with local arrays limited by locals_count.
        // it would have sacrificed performance
        for i in (0..self.locals_count).rev() {
            let local = &self.stack[i as usize];
            if local.depth <= self.scope_depth {
                break;
            }
            self.locals_count -= 1;
        }
        (previous_count - self.locals_count) as usize
    }
    /// Declares a local variable
    pub fn declare(&mut self, name: String) {
        self.stack[self.locals_count as usize] = Local {
            name,
            depth: self.scope_depth,
            initialized: false,
        };
        self.locals_count += 1;
    }

    /// Marks the local variable as initialized
    pub fn define(&mut self) {
        self.stack[self.locals_count as usize - 1].initialized = true;
        self.stack[self.locals_count as usize - 1].depth = self.scope_depth;
    }

    pub fn contains(&self, name: &str) -> bool {
        // TODO: Optimize, also limit to locals_count
        self.stack
            .iter()
            .rev()
            .any(|local| local.depth == self.scope_depth && local.name == name)
    }
    pub fn index_of(&self, name: &str) -> Option<(usize, bool)> {
        // Start with the most recent local and work backwards
        for i in (0..self.locals_count).rev() {
            let local = &self.stack[i as usize];
            if local.name == name {
                return Some((i as usize, local.initialized));
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        for i in 0..self.locals_count {
            let local = &self.stack[i as usize];
            println!("{:?}", local);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Local {
    name: String,
    depth: usize,
    initialized: bool,
}

impl Local {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            depth: 0,
            initialized: false,
        }
    }
}
