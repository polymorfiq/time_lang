mod state;
use state::State;
use regex::Regex;

static COMMENT_REGEX: &str = r"^(#+)(?<comment>.*)(#*)$";
static CMD_REGEX: &str = r"^(?<cmd>[a-zA-Z0-9_]+)([\s]+(?<args>.+))?;$";

pub struct Parser<'a> {
    filename: &'a str,
    state: State,
    source: String,
    lineno: usize
}

impl<'a> Parser<'a> {
    pub const fn new(filename: &'a str) -> Self {
        Self{
            filename: filename,
            state: State::General,
            source: String::new(),
            lineno: 0
        }
    }

    pub fn parse_line(&mut self, line: String) {
        self.lineno += 1;
        let cmd_re = Regex::new(CMD_REGEX).unwrap();
        let comment_re = Regex::new(COMMENT_REGEX).unwrap();

        if let Some(cmd) = cmd_re.captures(&line) {
            let args: Vec<&str> = cmd["args"].split(",").collect();

            match (&cmd["cmd"], &args[..]) {
                ("defalphabet", [name]) => self.start_state(State::alphabet(name.to_string())),
                ("defclock", [name]) => self.start_state(State::clock(name.to_string())),
                ("defprogram", [name]) => self.start_state(State::program(name.to_string())),
                (cmd, args) => {
                    self.state.process_command(self.filename, self.lineno, cmd, args);
                }
            }
        } else if let Some(_comment) = comment_re.captures(&line) {
            ()
        }
    }

    pub fn generate(&self) -> String {
        self.source.clone()
    }

    fn start_state(&mut self, state: State) {
        let result: String = self.state.generate();
        self.source.push_str(result.as_str());
        self.source.push_str("\n");

        self.state = state;
    }
}