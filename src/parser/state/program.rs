use quote::{format_ident, quote};
use convert_case::{Case, Casing};

#[derive(Debug)]
pub enum ArgType {
    Name(String),
    Label(String),
    Gateway(String),
    Exit(String),
    Alphabet(String),
    Clock(String),
    Moment(String),
    Character(String),
    Number(String),
    Program(String)
}

#[derive(Debug)]
pub enum Instruction {
    StartMoment(ArgType, ArgType),
    PushMoment(ArgType, ArgType),
    ForwardMoment(ArgType, ArgType),
    PushChar(ArgType, ArgType),
    PushVal(ArgType, ArgType),
    JumpEarlier(ArgType, ArgType, ArgType),
    JumpLater(ArgType, ArgType, ArgType),
    ForwardDuration(ArgType, ArgType),
    Connect(ArgType, ArgType),
    ExitGateway(ArgType, ArgType)
}

#[derive(Debug)]
pub struct Program {
    name: String,
    instructions: Vec<(ArgType, Vec<Instruction>)>,
    gateways: Vec<(ArgType, ArgType, ArgType, ArgType)>,
    exits: Vec<(ArgType, ArgType, ArgType, ArgType)>
}

impl Program {
    pub const fn new(name: String) -> Self {
        Self{
            name: name,
            instructions: vec![],
            gateways: vec![],
            exits: vec![]
        }
    }

    pub fn process_command(&mut self, filename: &str, lineno: usize, cmd: &str, args: &[&str]) {
        if self.instructions.len() == 0 {
            self.instructions.push((ArgType::Name("root".to_string()), vec![]));
        }

        let latest_func = self.instructions.last_mut().unwrap();

        match (cmd, args) {
            ("start_moment", [moment, exit]) => {
                latest_func.1.push(Instruction::StartMoment(ArgType::Moment(moment.to_string()), ArgType::Exit(exit.to_string())));
            },

            ("reg_gateway", [name, alphabet, clock, buf_size]) => {
                self.gateways.push((ArgType::Name(name.to_string()), ArgType::Alphabet(alphabet.to_string()), ArgType::Clock(clock.to_string()), ArgType::Number(buf_size.to_string())));
            },

            ("reg_exit", [name, alphabet, clock, buf_size]) => {
                self.exits.push((ArgType::Name(name.to_string()), ArgType::Alphabet(alphabet.to_string()), ArgType::Clock(clock.to_string()), ArgType::Number(buf_size.to_string())));
            },

            ("reg_exit_gateway", [connected_name, gateway]) => {
                latest_func.1.push(Instruction::ExitGateway(ArgType::Exit(connected_name.to_string()), ArgType::Gateway(gateway.to_string())));
            },

            ("label", [name]) => {
                self.instructions.push((ArgType::Name(name.to_string()), vec![]));
            },

            ("jump_earlier", [label_name, a, b]) => {
                latest_func.1.push(Instruction::JumpEarlier(ArgType::Label(label_name.to_string()), ArgType::Gateway(a.to_string()), ArgType::Gateway(b.to_string())));
            },

            ("jump_later", [label_name, a, b]) => {
                latest_func.1.push(Instruction::JumpLater(ArgType::Label(label_name.to_string()), ArgType::Gateway(a.to_string()), ArgType::Gateway(b.to_string())));
            },

            ("push_moment", [moment_incr, exit]) => {
                latest_func.1.push(Instruction::PushMoment(ArgType::Moment(moment_incr.to_string()), ArgType::Exit(exit.to_string())));
            },

            ("forward_moment", [gateway, exit]) => {
                latest_func.1.push(Instruction::ForwardMoment(ArgType::Gateway(gateway.to_string()), ArgType::Exit(exit.to_string())));
            },

            ("push_char", [chr, exit]) => {
                latest_func.1.push(Instruction::PushChar(ArgType::Character(chr.to_string()), ArgType::Exit(exit.to_string())));
            },

            ("push_val", [chr, exit]) => {
                latest_func.1.push(Instruction::PushVal(ArgType::Number(chr.to_string()), ArgType::Exit(exit.to_string())));
            },

            ("forward_duration", [gateway, exit]) => {
                latest_func.1.push(Instruction::ForwardDuration(ArgType::Gateway(gateway.to_string()), ArgType::Exit(exit.to_string())));
            },

            ("connect", [program, name]) => {
                latest_func.1.push(Instruction::Connect(ArgType::Program(program.to_string()), ArgType::Name(name.to_string())));
            },

            _ => {
                panic!("{}:{} Program ({}) - unknown command: {} ({:?})", filename, lineno, self.name, cmd, args);
            }
        }
    }

    pub fn gateway_field(&self, name: &String, alphabet: &String, clock: &String, buf_size: &String) -> proc_macro2::TokenStream {
        let field_name = format_ident!("gateway_{}", name.to_case(Case::Snake));
        let alphabet_name = format_ident!("Alphabet{}", alphabet.to_case(Case::Pascal));
        let clock_name = format_ident!("Clock{}", clock.to_case(Case::Pascal));
        let buf_size_lit: proc_macro2::TokenStream = buf_size.parse().unwrap();

        quote! {
            pub #field_name: Stream<#alphabet_name, #clock_name, #buf_size_lit>,
        }
    }

    pub fn exit_field(&self, name: &String, alphabet: &String, clock: &String, buf_size: &String) -> proc_macro2::TokenStream {
        let field_name = format_ident!("exit_{}", name.to_case(Case::Snake));
        let alphabet_name = format_ident!("Alphabet{}", alphabet.to_case(Case::Pascal));
        let clock_name = format_ident!("Clock{}", clock.to_case(Case::Pascal));
        let buf_size_lit: proc_macro2::TokenStream = buf_size.parse().unwrap();

        quote! {
            pub #field_name: Stream<#alphabet_name, #clock_name, #buf_size_lit>,
        }
    }

    pub fn instruction_call(&self, instruction: &Instruction) -> proc_macro2::TokenStream {
        use Instruction::*;

        match instruction {
            StartMoment(ArgType::Moment(moment), ArgType::Exit(exit_name)) => {
                let exit_field = format_ident!("exit_{}", exit_name.to_case(Case::Snake));
                let moment_lit: proc_macro2::TokenStream = moment.parse().unwrap();

                quote! {
                    self.#exit_field.set_initial_moment(#moment_lit);
                }
            }
            
            PushMoment(ArgType::Moment(moment), ArgType::Exit(exit_name)) => {
                let exit_field = format_ident!("exit_{}", exit_name.to_case(Case::Snake));
                let moment_lit: proc_macro2::TokenStream = moment.parse().unwrap();
                let push_error = format!("Could not push_moment to Exit ({})", exit_name);

                quote! {
                    self.#exit_field.push_moment(#moment_lit).expect(#push_error);
                }
            }
            
            ForwardMoment(ArgType::Gateway(gateway_name), ArgType::Exit(exit_name)) => {
                let gateway_field = format_ident!("gateway_{}", gateway_name.to_case(Case::Snake));
                let exit_field = format_ident!("exit_{}", exit_name.to_case(Case::Snake));
                let push_moment_fail_msg = format!("Failed to forward moment from Gateway {} to Exit {}", gateway_name, exit_name);

                quote! {
                    if self.#gateway_field.next_is_moment() {
                        match self.#gateway_field.pop() {
                            StreamItem::Moment(moment) => {
                                self.#exit_field.push_moment(moment).expect(#push_moment_fail_msg);
                            }
                            _ => {
                                panic!("Unreachable Code - unexpectedly popped a non-moment when calling forward_moment()");
                            }
                        }
                    } else {
                        panic!("Tried to forward_moment from {} to {} when the next item in the gateway, is not a Moment", #gateway_name, #exit_name)
                    }
                }
            }

            PushVal(ArgType::Number(val), ArgType::Exit(exit_name)) => {
                let exit_field = format_ident!("exit_{}", exit_name.to_case(Case::Snake));
                let val_lit: proc_macro2::TokenStream = val.parse().unwrap();

                let alphabet = self.exits.iter().find_map(|(name, alphabet, _, _)| {
                    match (name, alphabet) {
                        (ArgType::Name(name), ArgType::Alphabet(alphabet)) if name == exit_name => Some(alphabet),
                        _ => None
                    }
                }).unwrap_or_else(|| {
                    panic!("Could not find Exit ({}) for Program ({})", exit_name, self.name);
                });
                let alphabet_name = format_ident!("Alphabet{}", alphabet.to_case(Case::Pascal));
                let error_message = format!("No character found in Alphabet ({}): {:?}", alphabet, val);
                let push_error = format!("Could not push_val to Exit ({})", exit_name);
                
                quote! {
                    self.#exit_field.push(#alphabet_name::to_char(#val_lit).expect(#error_message)).expect(#push_error);
                }
            }

            PushChar(ArgType::Character(chr), ArgType::Exit(exit_name)) => {
                let alphabet = self.exits.iter().find_map(|(name, alphabet, _, _)| {
                    match (name, alphabet) {
                        (ArgType::Name(name), ArgType::Alphabet(alphabet)) if name == exit_name => Some(alphabet),
                        _ => None
                    }
                }).unwrap_or_else(|| {
                    panic!("Could not find Exit ({}) for Program ({})", exit_name, self.name);
                });

                let alphabet_name = format_ident!("Alphabet{}", alphabet.to_case(Case::Pascal));
                let enum_name = format_ident!("{}", chr.to_case(Case::Pascal));
                let exit_field = format_ident!("exit_{}", exit_name.to_case(Case::Snake));
                let error_message = format!("Could not push_char ({:?})", chr);

                quote!{
                    self.#exit_field.push(<#alphabet_name as AlphabetLike>::CharEnum::#enum_name()).expect(#error_message);
                }
            },

            ForwardDuration(ArgType::Gateway(gateway_name), ArgType::Exit(exit_name)) => {
                let gateway_field = format_ident!("gateway_{}", gateway_name.to_case(Case::Snake));
                let exit_field = format_ident!("exit_{}", exit_name.to_case(Case::Snake));

                let push_fail_msg = format!("Failed to forward character from Gateway {} to Exit {}", gateway_name, exit_name);
                let push_moment_fail_msg = format!("Failed to forward moment from Gateway {} to Exit {}", gateway_name, exit_name);

                quote!{
                    loop {
                        match self.#gateway_field.pop() {
                            StreamItem::Character(chr) => {
                                self.#exit_field.push(chr).expect(#push_fail_msg);
                            }

                            StreamItem::Moment(moment) => {
                                self.#exit_field.push_moment(moment).expect(#push_moment_fail_msg);
                                break;
                            }

                            StreamItem::Empty => {
                                continue
                            }
                        }
                    }
                }
            },

            JumpEarlier(ArgType::Label(label), ArgType::Gateway(gateway_a), ArgType::Gateway(gateway_b)) => {
                let label_func = format_ident!("label_{}", label.to_case(Case::Snake));
                let gateway_a_field = format_ident!("gateway_{}", gateway_a.to_case(Case::Snake));
                let gateway_b_field = format_ident!("gateway_{}", gateway_b.to_case(Case::Snake));

                let clock_a = self.gateways.iter().find_map(|(name, _, clock, _)| {
                    match (name, clock) {
                        (ArgType::Name(name), ArgType::Clock(clock)) if name == gateway_a => Some(format_ident!("Clock{}", clock)),
                        _ => None
                    }
                }).unwrap_or_else(|| {
                    panic!("Could not find Gateway ({}) for Program ({})", gateway_a, self.name);
                });

                let clock_b = self.gateways.iter().find_map(|(name, _, clock, _)| {
                    match (name, clock) {
                        (ArgType::Name(name), ArgType::Clock(clock)) if name == gateway_b => Some(format_ident!("Clock{}", clock)),
                        _ => None
                    }
                }).unwrap_or_else(|| {
                    panic!("Could not find Gateway ({}) for Program ({})", gateway_b, self.name);
                });

                let clock_repr_error = format!("(Clock of) Gateway {} and (Clock of) Gateway {} being compared while not representing the same thing", gateway_a, gateway_b);

                quote! {
                    if #clock_a::represents() != #clock_b::represents() {
                        panic!(#clock_repr_error);
                    }

                    match (self.#gateway_a_field.current_moment(), self.#gateway_b_field.current_moment()) {
                        (None, Some(_)) => {
                            return self.#label_func();
                        }

                        (Some(a), Some(b)) if a < b => {
                            return self.#label_func();
                        }

                        _ => ()
                    }
                }
            },

            JumpLater(ArgType::Label(label), ArgType::Gateway(gateway_a), ArgType::Gateway(gateway_b)) => {
                let label_func = format_ident!("label_{}", label.to_case(Case::Snake));
                let gateway_a_field = format_ident!("gateway_{}", gateway_a.to_case(Case::Snake));
                let gateway_b_field = format_ident!("gateway_{}", gateway_b.to_case(Case::Snake));

                let clock_a = self.gateways.iter().find_map(|(name, _, clock, _)| {
                    match (name, clock) {
                        (ArgType::Name(name), ArgType::Clock(clock)) if name == gateway_a => Some(format_ident!("Clock{}", clock)),
                        _ => None
                    }
                }).unwrap_or_else(|| {
                    panic!("Could not find Gateway ({}) for Program ({})", gateway_a, self.name);
                });

                let clock_b = self.gateways.iter().find_map(|(name, _, clock, _)| {
                    match (name, clock) {
                        (ArgType::Name(name), ArgType::Clock(clock)) if name == gateway_b => Some(format_ident!("Clock{}", clock)),
                        _ => None
                    }
                }).unwrap_or_else(|| {
                    panic!("Could not find Gateway ({}) for Program ({})", gateway_b, self.name);
                });

                let clock_repr_error = format!("(Clock of) Gateway {} and (Clock of) Gateway {} being compared while not representing the same thing", gateway_a, gateway_b);

                quote! {
                    if #clock_a::represents() != #clock_b::represents() {
                        panic!(#clock_repr_error);
                    }

                    match (self.#gateway_a_field.current_moment(), self.#gateway_b_field.current_moment()) {
                        (Some(_), None) => {
                            return self.#label_func();
                        }

                        (Some(a), Some(b)) if a > b => {
                            return self.#label_func();
                        }

                        _ => ()
                    }
                }
            }

            instr => {
                let error_message = format!("Not implemented: {:?}", instr);

                quote!{
                    compile_error!(#error_message);
                }
            }
        }
    }

    pub fn func_def(&self, name: &String, instructions: &Vec<Instruction>) -> proc_macro2::TokenStream {
        let func_name = format_ident!("label_{}", name.to_case(Case::Snake));
        let instructions: Vec<_> = instructions.iter().map(|instruction| self.instruction_call(instruction)).collect();

        quote! {
            pub fn #func_name(&mut self) {
                #(#instructions)*
            }
        }
    }

    pub fn generate(&self) -> Result<String, String> {
        let struct_name = format_ident!("Program{}", self.name.to_case(Case::Pascal));
        let gateways: Vec<_> = self.gateways.iter().map(|gateway_data| {
            match gateway_data {
                (ArgType::Name(name), ArgType::Alphabet(alphabet), ArgType::Clock(clock), ArgType::Number(buf_size)) => {
                    self.gateway_field(name, alphabet, clock, buf_size)
                },

                _ => panic!("Unexpected reg_gateway params: {:?}", gateway_data)
            }
        }).collect();

        let exits: Vec<_> = self.exits.iter().map(|exit_data| {
            match exit_data {
                (ArgType::Name(name), ArgType::Alphabet(alphabet), ArgType::Clock(clock), ArgType::Number(buf_size)) => {
                    self.exit_field(name, alphabet, clock, buf_size)
                },

                _ => panic!("Unexpected reg_exit params: {:?}", exit_data)
            }
        }).collect();

        let funcs: Vec<_> = self.instructions.iter().map(|func_data| {
            match func_data {
                (ArgType::Name(name), instructions) => self.func_def(name, instructions),
                _ => panic!("Unexpected label data: {:?}", func_data)
            }
        }).collect();

        let formatted = rustfmt_wrapper::rustfmt(quote! {
            pub struct #struct_name {
                #(#gateways)*
                #(#exits)*
            }

            impl #struct_name {
                #(#funcs)*
            }
        });

        match formatted {
            Ok(formatted_str) => Ok(formatted_str),
            Err(rustfmt_wrapper::Error::Rustfmt(err)) => Err(format!("Error formatting Program({}):\n{}", self.name, err)),
            Err(err) => Err(format!("Error generating Program({}):\n{}", self.name, err))
        }
    }
}