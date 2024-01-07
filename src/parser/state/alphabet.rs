use quote::{format_ident, quote};
use convert_case::{Case, Casing};

#[derive(Debug)]
pub struct Alphabet {
    name: String,
    char_type: Option<String>,
    chars: Vec<(String, String)>
}

impl Alphabet {
    pub const fn new(name: String) -> Self {
        Self{name: name, char_type: None, chars: vec![]}
    }

    pub fn process_command(&mut self, filename: &str, lineno: usize, cmd: &str, args: &[&str]) {
        match (cmd, args) {
            ("set_char_type", [char_type]) => {
                self.char_type = Some(char_type.to_string());
            },

            ("def_char", [hex_rep, name]) => {
                self.chars.push((hex_rep.to_string(), name.to_string()));
            },
            
            _ => {
                panic!("{}:{} Alphabet ({}) - unknown command: {} ({:?})", filename, lineno, self.name, cmd, args);
            }
        }
    }

    pub fn generate(&self) -> Result<String, String> {
        let char_rep = format_ident!("{}", if let Some(ct) = self.char_type.as_ref() { ct.clone() } else {
            return Err(format!("Never called set_char_type on Alphabet ({})", self.name).to_string())
        });

        let char_rep_name = format_ident!("CharRep{}", self.name.to_case(Case::Pascal));
        let char_enum_name = format_ident!("Char{}", self.name.to_case(Case::Pascal));
        let struct_name = format_ident!("Alphabet{}", self.name.to_case(Case::Pascal));

        let char_enums: Vec<_> = self.chars.iter().map(|(_, char_name)| {
            let rep_enum = format_ident!("{}", char_name.to_case(Case::Pascal));

            quote!{
                #rep_enum(),
            }
        }).collect();

        let char_matches: Vec<_> = self.chars.iter().map(|(char_rep_val, char_name)| {
            let rep_enum = format_ident!("{}", char_name.to_case(Case::Pascal));
            let lit_rep: proc_macro2::TokenStream = char_rep_val.parse().unwrap();

            quote!{
                #lit_rep => Ok(#rep_enum()),
            }
        }).collect();

        let formatted = rustfmt_wrapper::rustfmt(quote! {
            type #char_rep_name = #char_rep;
            pub enum #char_enum_name {
                #(#char_enums)*
            }

            pub struct #struct_name {}
            impl #struct_name {
                const fn to_char(rep: #char_rep) -> Result<#char_enum_name, AlphabetError<#char_rep>> {
                    use #char_enum_name::*;
                    
                    
                    match rep {
                        #(#char_matches)*
                        _ => Err(AlphabetError::UnknownCharacter(rep))
                    }
                }
            }

            impl AlphabetLike<#char_rep, #char_enum_name> for #struct_name {
                fn to_char(rep: #char_rep) -> Result<#char_enum_name, AlphabetError<#char_rep>> {
                    <#struct_name>::to_char(rep)
                }
            }
        });

        match formatted {
            Ok(formatted_str) => Ok(formatted_str),
            err => Err(format!("Error generating Alphabet({}):\n{:?}", self.name, err))
        }
    }
}