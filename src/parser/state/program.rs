use quote::{format_ident, quote};
use convert_case::{Case, Casing};

#[derive(Debug)]
pub enum Instruction {
    StartMoment(String, String),
    PushMoment(String, String),
    PushChar(String, String),
    Label(String),
    JumpLessThan(String, String, String),
    JumpGreaterThan(String, String, String),
    ForwardDuration(String, String),
    Connect(String, String),
    ExitGateway(String, String)
}

#[derive(Debug)]
pub struct Program {
    name: String,
    instructions: Vec<(String, Vec<Instruction>)>,
    gateways: Vec<(String, String, String, String)>,
    exits: Vec<(String, String, String, String)>
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
            self.instructions.push(("root".to_string(), vec![]));
        }

        let latest_func = self.instructions.last_mut().unwrap();

        match (cmd, args) {
            ("start_moment", [moment, exit]) => {
                latest_func.1.push(Instruction::StartMoment(moment.to_string(), exit.to_string()));
            },

            ("reg_gateway", [name, alphabet, clock, buf_size]) => {
                self.gateways.push((name.to_string(), alphabet.to_string(), clock.to_string(), buf_size.to_string()));
            },

            ("reg_exit", [name, alphabet, clock, buf_size]) => {
                self.exits.push((name.to_string(), alphabet.to_string(), clock.to_string(), buf_size.to_string()));
            },

            ("reg_exit_gateway", [connected_name, gateway]) => {
                latest_func.1.push(Instruction::ExitGateway(connected_name.to_string(), gateway.to_string()));
            },

            ("label", [name]) => {
                latest_func.1.push(Instruction::Label(name.to_string()));
            },

            ("jlt", [label_name, a, b]) => {
                latest_func.1.push(Instruction::JumpLessThan(label_name.to_string(), a.to_string(), b.to_string()));
            },

            ("jgt", [label_name, a, b]) => {
                latest_func.1.push(Instruction::JumpGreaterThan(label_name.to_string(), a.to_string(), b.to_string()));
            },

            ("push_moment", [moment_incr, exit]) => {
                latest_func.1.push(Instruction::PushMoment(moment_incr.to_string(), exit.to_string()));
            },

            ("push_char", [chr, exit]) => {
                latest_func.1.push(Instruction::PushChar(chr.to_string(), exit.to_string()));
            },

            ("forward_duration", [gateway, exit]) => {
                latest_func.1.push(Instruction::ForwardDuration(gateway.to_string(), exit.to_string()));
            },

            ("connect", [program, name]) => {
                latest_func.1.push(Instruction::Connect(program.to_string(), name.to_string()));
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
            Instruction::PushChar(chr, exit_name) => {
                let exit_field = format_ident!("exit_{}", exit_name.to_case(Case::Snake));

                quote!{
                    self.#exit_field.push_with_name(#chr).expect("Could not push #chr");
                }
            },

            _ => quote!{}
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
        let gateways: Vec<_> = self.gateways.iter().map(|(name, alphabet, clock, buf_size)| self.gateway_field(name, alphabet, clock, buf_size)).collect();
        let exits: Vec<_> = self.exits.iter().map(|(name, alphabet, clock, buf_size)| self.exit_field(name, alphabet, clock, buf_size)).collect();
        let funcs: Vec<_> = self.instructions.iter().map(|(name, instructions)| self.func_def(name, instructions)).collect();

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