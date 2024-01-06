pub struct Alphabet {
    name: String,
    bits: Option<u64>,
    chars: Vec<(String, String)>
}

impl Alphabet {
    pub const fn new(name: String) -> Self {
        Self{name: name, bits: None, chars: vec![]}
    }

    pub fn process_command(&mut self, filename: &str, lineno: usize, cmd: &str, args: &[&str]) {
        match (cmd, args) {
            ("set_alphabet_bits", [num_bits]) => {
                let num_bits = num_bits.parse().expect("num_bits needs to be a u64");
                if num_bits % 8 != 0 {
                    panic!("{}:{} set_alphabet_bits ({} invalid) - Must be divisible by 8", filename, lineno, num_bits);
                }

                self.bits = Some(num_bits);
            },

            ("def_char", [hex_rep, name]) => {
                self.chars.push((hex_rep.to_string(), name.to_string()));
            },
            
            _ => {
                panic!("{}:{} Alphabet ({}) - unknown command: {} ({:?})", filename, lineno, self.name, cmd, args);
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