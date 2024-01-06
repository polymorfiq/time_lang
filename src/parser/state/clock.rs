pub struct Clock {
    name: String,
    bits: Option<u64>,
    repr: Option<String>
}

impl Clock {
    pub const fn new(name: String) -> Self {
        Self{name: name, bits: None, repr: None}
    }

    pub fn process_command(&mut self, filename: &str, lineno: usize, cmd: &str, args: &[&str]) {
        match (cmd, args) {
            ("set_clock_bits", [num_bits]) => {
                let num_bits = num_bits.parse().expect("num_bits needs to be a u64");
                if num_bits % 8 != 0 {
                    panic!("{}:{} set_clock_bits ({} invalid) - Must be divisible by 8", filename, lineno, num_bits);
                }

                self.bits = Some(num_bits);
            },

            ("set_clock_repr", [repr]) => {
                self.repr = Some(repr.to_string());
            },

            _ => {
                panic!("{}:{} Clock ({}) - unknown command: {} ({:?})", filename, lineno, self.name, cmd, args);
            }
        }
    }

    pub fn generate(&self) -> String {
        "".to_string()
    }
}