use quote::{format_ident, quote};
use convert_case::{Case, Casing};

#[derive(Debug)]
enum Instruction {
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
    instructions: Vec<Instruction>,
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
        match (cmd, args) {
            ("start_moment", [moment, exit]) => {
                self.instructions.push(Instruction::StartMoment(moment.to_string(), exit.to_string()));
            },

            ("reg_gateway", [name, alphabet, clock, buf_size]) => {
                self.gateways.push((name.to_string(), alphabet.to_string(), clock.to_string(), buf_size.to_string()));
            },

            ("reg_exit", [name, alphabet, clock, buf_size]) => {
                self.exits.push((name.to_string(), alphabet.to_string(), clock.to_string(), buf_size.to_string()));
            },

            ("reg_exit_gateway", [connected_name, gateway]) => {
                self.instructions.push(Instruction::ExitGateway(connected_name.to_string(), gateway.to_string()));
            },

            ("label", [name]) => {
                self.instructions.push(Instruction::Label(name.to_string()));
            },

            ("jlt", [label_name, a, b]) => {
                self.instructions.push(Instruction::JumpLessThan(label_name.to_string(), a.to_string(), b.to_string()));
            },

            ("jgt", [label_name, a, b]) => {
                self.instructions.push(Instruction::JumpGreaterThan(label_name.to_string(), a.to_string(), b.to_string()));
            },

            ("push_moment", [moment_incr, exit]) => {
                self.instructions.push(Instruction::PushMoment(moment_incr.to_string(), exit.to_string()));
            },

            ("push_char", [chr, exit]) => {
                self.instructions.push(Instruction::PushChar(chr.to_string(), exit.to_string()));
            },

            ("forward_duration", [gateway, exit]) => {
                self.instructions.push(Instruction::ForwardDuration(gateway.to_string(), exit.to_string()));
            },

            ("connect", [program, name]) => {
                self.instructions.push(Instruction::Connect(program.to_string(), name.to_string()));
            },

            _ => {
                panic!("{}:{} Program ({}) - unknown command: {} ({:?})", filename, lineno, self.name, cmd, args);
            }
        }
    }

    pub fn gateway_field(&self, name: &String, alphabet: &String, clock: &String, buf_size: &String) -> proc_macro2::TokenStream {
        let field_name = format_ident!("gateway_{}", name.to_case(Case::Snake));
        let char_rep_name = format_ident!("CharRep{}", alphabet.to_case(Case::Pascal));
        let moment_rep_name = format_ident!("ClockRep{}", clock.to_case(Case::Pascal));
        let buf_size_lit: proc_macro2::TokenStream = buf_size.parse().unwrap();

        quote! {
            pub #field_name: Stream<#char_rep_name, #moment_rep_name, #buf_size_lit>,
        }
    }

    pub fn generate(&self) -> Result<String, String> {
        let struct_name = format_ident!("Program{}", self.name.to_case(Case::Pascal));
        let gateways: Vec<_> = self.gateways.iter().map(|(name, alphabet, clock, buf_size)| self.gateway_field(name, alphabet, clock, buf_size)).collect();

        let formatted = rustfmt_wrapper::rustfmt(quote! {
            pub struct #struct_name {
                #(#gateways)*
            }
        });

        match formatted {
            Ok(formatted_str) => Ok(formatted_str),
            err => Err(format!("Error generating Program({}):\n{:?}", self.name, err))
        }
    }
}