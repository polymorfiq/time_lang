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
            panic!("Error writing Header base code:\n{}", val);
        });

        let alphabet_code = rustfmt_wrapper::rustfmt(quote! {
            #[derive(Debug)]
            pub enum AlphabetError<CharRep: Debug> {
                UnknownCharacter(CharRep),
                UnexpectedError(&'static str),
                NameNotFound()
            }

            pub trait AlphabetLike {
                type CharRep: Copy + Clone + Debug;
                type CharEnum: Copy + Clone + Debug;

                fn char_with_name(rep: &str) -> Result<Self::CharEnum, AlphabetError<&str>>;
                fn to_char(rep: Self::CharRep) -> Result<Self::CharEnum, AlphabetError<Self::CharRep>>;
                fn to_val(rep: Self::CharEnum) -> Self::CharRep;
            }
        }).unwrap_or_else(|val| {
            panic!("Error writing Alphabet base code:\n{}", val);
        });

        let clock_code = rustfmt_wrapper::rustfmt(quote! {
            pub enum ClockMoment<MomentRep> {
                UnixSeconds(MomentRep),
                UnixMilliseconds(MomentRep),
                Quantity(MomentRep)
            }

            pub trait ClockLike {
                type MomentRep: Copy + Clone + Debug;

                fn represents(&self) -> &str;
                fn to_moment(rep: Self::MomentRep) -> ClockMoment<Self::MomentRep>;
            }

            pub trait AddableClockLike<MomentRep: core::ops::Add<Output = MomentRep>> {
                fn add(moment: ClockMoment<MomentRep>, rep: MomentRep) -> ClockMoment<MomentRep> {
                    match moment {
                        ClockMoment::Quantity(orig_rep) => ClockMoment::Quantity(orig_rep + rep),
                        ClockMoment::UnixMilliseconds(orig_rep) => ClockMoment::UnixMilliseconds(orig_rep + rep),
                        ClockMoment::UnixSeconds(orig_rep) => ClockMoment::UnixSeconds(orig_rep + rep)
                    }
                }
            }
            
        }).unwrap_or_else(|val| {
            panic!("Error writing Clock base code:\n{}", val);
        });

        let stream_code = rustfmt_wrapper::rustfmt(quote! {
            #[derive(Debug)]
            pub enum ExitError {
                BufferFull
            }
            
            pub trait ExitLike<Alphabet: AlphabetLike, Clock: ClockLike> {
                type InternalItem;
                type Item;

                fn set_initial_moment(&mut self, monent: Clock::MomentRep);
                fn accepting_pushes(&mut self) -> bool;
                fn push(&mut self, chr: Alphabet::CharEnum) -> Result<(), ExitError>;
                fn push_moment(&mut self, moment: Clock::MomentRep) -> Result<(), ExitError>;

                fn push_with_name(&mut self, chr_name: &str) -> Result<(), ExitError> {
                    self.push(Alphabet::char_with_name(chr_name).unwrap_or_else(|_| { panic!("Unknown char name: {}", chr_name)}))
                }
            }

            pub trait GatewayLike<Alphabet: AlphabetLike, Clock: ClockLike, const BUFFER_SIZE: usize> {
                type InternalItem;
                type Item;
                
                fn pop(&mut self) -> Self::Item;
                fn forward_duration<Exit: ExitLike<Alphabet, Clock>>(&mut self, exit: &mut Exit) -> Result<(), ExitError>;
                fn current_moment(&self) -> Option<Clock::MomentRep>;
                fn is_empty(&self) -> bool;
                fn next_is_character(&self) -> bool;
                fn next_is_moment(&self) -> bool;
            }

            #[derive(Copy, Clone, Debug)]
            pub enum StreamItem<CharacterRep, Moment> {
                Empty,
                Character(CharacterRep),
                Moment(Moment)
            }

            impl<CharacterRep, Moment> Default for StreamItem<CharacterRep, Moment> {
                fn default() -> Self { Self::Empty }
            }

            pub struct Stream<Alphabet: AlphabetLike, Clock: ClockLike, const BUFFER_SIZE: usize> {
                buffer: [StreamItem<Alphabet::CharRep, Clock::MomentRep>; BUFFER_SIZE],
                idx: usize,
                buffered_total: usize,
                buffered_moments: usize,
                buffered_characters: usize,
                last_seen_moment: Option<Clock::MomentRep>
            }

            impl<Alphabet: AlphabetLike, Clock: ClockLike, const BUFFER_SIZE: usize> Stream<Alphabet, Clock, BUFFER_SIZE> {
                fn inc_index(&mut self) {
                    self.idx = (self.idx + 1) % BUFFER_SIZE;
                }
            }

            impl<Alphabet: AlphabetLike, Clock: ClockLike, const BUFFER_SIZE: usize> ExitLike<Alphabet, Clock> for Stream<Alphabet, Clock, BUFFER_SIZE> {
                type InternalItem = StreamItem<Alphabet::CharRep, Clock::MomentRep>;
                type Item = StreamItem<Alphabet::CharEnum, Clock::MomentRep>;

                fn set_initial_moment(&mut self, moment: Clock::MomentRep) {
                    self.last_seen_moment = Some(moment);
                }

                fn accepting_pushes(&mut self) -> bool { self.buffered_total < BUFFER_SIZE }

                fn push(&mut self, chr: Alphabet::CharEnum) -> Result<(), ExitError> {
                    if self.accepting_pushes() {
                        self.buffer[self.idx] = Self::InternalItem::Character(Alphabet::to_val(chr));
                        self.buffered_characters += 1;
                        self.buffered_total += 1;

                        self.inc_index();
                        Ok(())
                    } else {
                        Err(ExitError::BufferFull)
                    }
                }

                fn push_moment(&mut self, moment: Clock::MomentRep) -> Result<(), ExitError> {
                    if self.accepting_pushes() {
                        self.buffer[self.idx] = Self::InternalItem::Moment(moment);
                        self.buffered_moments += 1;
                        self.buffered_total += 1;

                        self.inc_index();
                        Ok(())
                    } else {
                        Err(ExitError::BufferFull)
                    }
                }
            }

            impl<Alphabet: AlphabetLike, Clock: ClockLike, const BUFFER_SIZE: usize> GatewayLike<Alphabet, Clock, BUFFER_SIZE> for Stream<Alphabet, Clock, BUFFER_SIZE> {
                type InternalItem = StreamItem<Alphabet::CharRep, Clock::MomentRep>;
                type Item = StreamItem<Alphabet::CharEnum, Clock::MomentRep>;

                fn pop(&mut self) -> Self::Item {
                    let last = core::mem::take(&mut self.buffer[self.idx]);
                    self.inc_index();

                    match last {
                        Self::InternalItem::Character(chr) => {
                            self.buffered_characters -= 1;
                            self.buffered_total -= 1;
                            Self::Item::Character(Alphabet::to_char(chr).unwrap_or_else(|err| {
                                panic!("Unexpected character received in stream: {:?}", err);
                            }))
                        },

                        Self::InternalItem::Moment(moment) => {
                            self.buffered_moments -= 1;
                            self.buffered_total -= 1;
                            self.last_seen_moment = Some(moment);
                            Self::Item::Moment(moment)
                        },
                        
                        Self::InternalItem::Empty => Self::Item::Empty
                    }
                }
                
                fn forward_duration<Exit: ExitLike<Alphabet, Clock>>(&mut self, exit: &mut Exit) -> Result<(), ExitError> {
                    while self.next_is_character() {
                        match self.pop() {
                            Self::Item::Character(chr) => {
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

                fn current_moment(&self) -> Option<Clock::MomentRep> {
                    self.last_seen_moment
                }

                fn is_empty(&self) -> bool {
                    self.buffered_total == 0
                }

                fn next_is_character(&self) -> bool {
                    match self.buffer[self.idx] {
                        Self::InternalItem::Character(_) => true,
                        _ => false
                    }
                }

                fn next_is_moment(&self) -> bool {
                    match self.buffer[self.idx] {
                        Self::InternalItem::Moment(_) => true,
                        _ => false
                    }
                }
            }
        }).unwrap_or_else(|val| {
            panic!("Error writing Stream base code:\n{}", val);
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
                panic!("Error generating code:\n{}\n\n{:?}", err, state);
            }
        }

        self.state = state;
    }
}