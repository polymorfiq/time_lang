mod state;
use state::State;
use regex::Regex;
use quote::quote;

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

    pub fn generate(&self) -> Result<String, String> {
        let header_code = rustfmt_wrapper::rustfmt(quote! {
            #![no_std]
        }).unwrap_or_else(|val| {
            panic!("Error writing Alphabet base code:\n{:?}", val);
        });

        let alphabet_code = rustfmt_wrapper::rustfmt(quote! {
            pub enum AlphabetError<CharRep> {
                UnknownCharacter(CharRep),
                UnexpectedError(&'static str)
            }

            pub trait AlphabetLike<CharRep, CharVal> {
                fn to_char(rep: CharRep) -> Result<CharVal, AlphabetError<CharRep>>;
            }
        }).unwrap_or_else(|val| {
            panic!("Error writing Alphabet base code:\n{:?}", val);
        });

        let clock_code = rustfmt_wrapper::rustfmt(quote! {
            pub enum ClockMoment<MomentRep> {
                Quantity(MomentRep)
            }

            pub trait ClockLike<MomentRep> {
                fn to_moment(rep: MomentRep) -> ClockMoment<MomentRep>;
            }

            pub trait AddableClockLike<MomentRep: core::ops::Add<Output = MomentRep>> {
                fn add(moment: ClockMoment<MomentRep>, rep: MomentRep) -> ClockMoment<MomentRep> {
                    match moment {
                        ClockMoment::Quantity(orig_rep) => ClockMoment::Quantity(orig_rep + rep)
                    }
                }
            }
            
        }).unwrap_or_else(|val| {
            panic!("Error writing Alphabet base code:\n{:?}", val);
        });

        let mut code = header_code.to_string();
        code.push_str(format!("\n{}", alphabet_code).as_str());
        code.push_str(format!("\n{}", clock_code).as_str());
        code.push_str(format!("\n{}", self.source).as_str());
        
        Ok(code)
    }

    fn start_state(&mut self, state: State) {

        match self.state.generate() {
            Ok(generated_code) => {
                self.source.push_str(generated_code.as_str());
                self.source.push_str("\n");
            },

            Err(err) => {
                panic!("Error generating code:\n{:?}\n\n{:?}", err, state);
            }
        }

        self.state = state;
    }
}