pub struct Alphabet {
    name: String,
    bits: Option<u64>
}

impl Alphabet {
    pub const fn new(name: String) -> Self {
        Self{name: name, bits: None}
    }

    pub fn process_command(&mut self, cmd: &str, args: &[&str]) {
        match (cmd, args) {
            ("set_alphabet_bits", [num_bits]) => {
                self.bits = Some(num_bits.parse().expect("num_bits needs to be a u64"));
            },
            
            _ => {
                panic!("Alphabet ({}) - unknown command: {} ({:?})", self.name, cmd, args);
            }
        }
    }

    pub fn generate(&self) -> String {
        format!(r#"
        const fn generate_alphabet_{name}() -> Alphabet {{
            Alphabet::new()
        }}
        
        static ALPHABET_{name}: Alphabet = generate_alphabet_{name}();
        "#, name = self.name)
    }
}