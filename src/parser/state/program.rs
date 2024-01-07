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
    PushChar(ArgType, ArgType),
    PushVal(ArgType, ArgType),
    Label(ArgType),
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
                latest_func.1.push(Instruction::Label(ArgType::Name(name.to_string())));
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

            ("push_char", [chr, exit]) => {
                latest_func.1.push(Instruction::PushChar(ArgType::Character(chr.to_string()), ArgType::Exit(exit.to_string())));
            },

            ("push_val", [chr, exit]) => {
                latest_func.1.push(Instruction::PushVal(ArgType::Character(chr.to_string()), ArgType::Exit(exit.to_string())));
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
        match instruction {
            Instruction::PushChar(ArgType::Character(chr), ArgType::Exit(exit_name)) => {
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

            instr => {
                let error_message = format!("Not implemented: {:?}", instr);

                quote!{
                    compile_error!(#error_message);
                }
            }
        }
    }

    pub fn func_def(&self, name: &String, instructions: &Vec<Instruction>) -> proc_macro2::TokenStream {
        let func_name = format_ident!("{}", name.to_case(Case::Snake));
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