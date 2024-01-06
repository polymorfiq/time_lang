pub struct Clock {
    name: String
}

impl Clock {
    pub const fn new(name: String) -> Self {
        Self{name: name}
    }

    pub fn process_command(&mut self, cmd: &str, args: &[&str]) {
        panic!("Clock ({}) - unknown command: {} ({:?})", self.name, cmd, args);
    }

    pub fn generate(&self) -> String {
        "".to_string()
    }
}