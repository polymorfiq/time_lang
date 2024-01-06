mod alphabet;
mod clock;
mod program;

pub enum State {
    General,
    Alphabet(alphabet::Alphabet),
    Clock(clock::Clock),
    Program(program::Program)
}

impl State {
    pub const fn alphabet(name: String) -> Self { Self::Alphabet(alphabet::Alphabet::new(name)) }
    pub const fn clock(name: String) -> Self { Self::Clock(clock::Clock::new(name)) }
    pub const fn program(name: String) -> Self { Self::Program(program::Program::new(name)) }

    pub fn generate(&self) -> String {
        use State::*;

        match self {
            General => "".to_string(),
            Alphabet(alphabet) => alphabet.generate(),
            Clock(clock) => clock.generate(),
            Program(prog) => prog.generate(),
        }
    }

    pub fn process_command(&mut self, cmd: &str, args: &[&str]) {
        use State::*;

        match self {
            General => panic!("General - Unknown command: {} ({:?})", cmd, args),
            Alphabet(alphabet) => alphabet.process_command(cmd, args),
            Clock(clock) => clock.process_command(cmd, args),
            Program(prog) => prog.process_command(cmd, args),
        }
    }
}