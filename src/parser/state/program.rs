pub struct Program {
    name: String
}

impl Program {
    pub const fn new(name: String) -> Self {
        Self{name: name}
    }

    pub fn process_command(&mut self, cmd: &str, args: &[&str]) {
        panic!("Program ({}) - unknown command: {} ({:?})", self.name, cmd, args);
    }

    pub fn generate(&self) -> String {
        "".to_string()
    }
}