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

            use core::default::Default;
            use core::fmt::Debug;
        }).unwrap_or_else(|val| {
            panic!("Error writing Header base code:\n{:?}", val);
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
            panic!("Error writing Clock base code:\n{:?}", val);
        });

        let stream_code = rustfmt_wrapper::rustfmt(quote! {
            pub enum ExitError {
                BufferFull
            }
            
            pub trait ExitLike<CharacterRep, Moment> {
                fn accepting_pushes(&mut self) -> bool;
                fn push(&mut self, chr: CharacterRep) -> Result<(), ExitError>;
                fn push_moment(&mut self, moment: Moment) -> bool;
            }

            pub trait GatewayLike<CharacterRep: Copy + Debug, Moment: Copy + Debug, const BUFFER_SIZE: usize> {
                fn pop(&mut self) -> StreamItem<CharacterRep, Moment>;
                fn forward_duration<Exit: ExitLike<CharacterRep, Moment>>(&mut self, exit: &mut Exit) -> Result<(), ExitError>;
                fn current_moment(&self) -> Option<Moment>;
                fn next_is_character(&self) -> bool;
                fn next_is_moment(&self) -> bool;
            }

            #[derive(Copy, Clone, Debug)]
            pub enum StreamItem<CharacterRep: Copy + Debug, Moment: Copy + Debug> {
                Empty,
                Character(CharacterRep),
                Moment(Moment)
            }

            impl<CharacterRep: Copy + Debug, Moment: Copy + Debug> Default for StreamItem<CharacterRep, Moment> {
                fn default() -> Self {
                    Self::Empty
                }
            }

            pub struct Stream<CharacterRep: Copy + Debug, Moment: Copy + Debug, const BUFFER_SIZE: usize> {
                buffer: [StreamItem<CharacterRep, Moment>; BUFFER_SIZE],
                idx: usize,
                buffered_total: usize,
                buffered_moments: usize,
                buffered_characters: usize,
                last_seen_moment: Option<Moment>
            }

            impl<CharacterRep: Copy + Debug, Moment: Copy + Debug, const BUFFER_SIZE: usize> Stream<CharacterRep, Moment, BUFFER_SIZE> {
                fn inc_index(&mut self) {
                    self.idx = (self.idx + 1) % BUFFER_SIZE;
                }
            }

            impl<CharacterRep: Copy + Debug, Moment: Copy + Debug, const BUFFER_SIZE: usize> GatewayLike<CharacterRep, Moment, BUFFER_SIZE> for Stream<CharacterRep, Moment, BUFFER_SIZE> {
                fn pop(&mut self) -> StreamItem<CharacterRep, Moment> {
                    let last = core::mem::take(&mut self.buffer[self.idx]);
                    self.inc_index();

                    match last {
                        StreamItem::Character(chr) => {
                            self.buffered_characters -= 1;
                            self.buffered_total -= 1;
                            StreamItem::Character(chr)
                        },

                        StreamItem::Moment(moment) => {
                            self.buffered_moments -= 1;
                            self.buffered_total -= 1;
                            self.last_seen_moment = Some(moment);
                            StreamItem::Moment(moment)
                        },
                        
                        StreamItem::Empty => StreamItem::Empty
                    }
                }
                
                fn forward_duration<Exit: ExitLike<CharacterRep, Moment>>(&mut self, exit: &mut Exit) -> Result<(), ExitError> {
                    while self.next_is_character() {
                        match self.pop() {
                            StreamItem::Character(chr) => {
                                let result = exit.push(chr);
                                match result {
                                    Ok(_) => (),
                                    Err(err) => {
                                        return Err(err)
                                    }
                                }
                            },

                            item => panic!("Expected to pop Character off Gateway. Popped something else:\n{:?}", item)
                        }
                    };
                    
                    Ok(())
                }

                fn current_moment(&self) -> Option<Moment> {
                    self.last_seen_moment
                }

                fn next_is_character(&self) -> bool {
                    match self.buffer[self.idx] {
                        StreamItem::Character(_) => true,
                        _ => false
                    }
                }

                fn next_is_moment(&self) -> bool {
                    match self.buffer[self.idx] {
                        StreamItem::Moment(_) => true,
                        _ => false
                    }
                }
            }
        }).unwrap_or_else(|val| {
            panic!("Error writing Stream base code:\n{:?}", val);
        });

        let mut code = header_code.to_string();
        code.push_str(format!("\n{}", alphabet_code).as_str());
        code.push_str(format!("\n{}", clock_code).as_str());
        code.push_str(format!("\n{}", stream_code).as_str());
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