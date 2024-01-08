use quote::{format_ident, quote};
use convert_case::{Case, Casing};

#[derive(Debug)]
pub struct Clock {
    name: String,
    moment_type: Option<String>,
    repr: Option<String>
}

impl Clock {
    pub const fn new(name: String) -> Self {
        Self{name: name, moment_type: None, repr: None}
    }

    pub fn process_command(&mut self, filename: &str, lineno: usize, cmd: &str, args: &[&str]) {
        match (cmd, args) {
            ("set_moment_type", [moment_type]) => {
                self.moment_type = Some(moment_type.to_string());
            },

            ("set_clock_repr", [repr]) => {
                self.repr = Some(repr.to_string());
            },

            _ => {
                panic!("{}:{} Clock ({}) - unknown command: {} ({:?})", filename, lineno, self.name, cmd, args);
            }
        }
    }

    pub fn generate(&self) -> Result<String, String> {
        let moment_enum = format_ident!("{}", if let Some(repr) = self.repr.as_ref() { repr.clone() } else {
            return Err(format!("Never called set_clock_repr on Clock ({})", self.name).to_string())
        }.to_case(Case::Pascal));
        let repr_name = self.repr.as_ref().unwrap();

        let struct_name = format_ident!("Clock{}", self.name.to_case(Case::Pascal));

        let moment_rep = format_ident!("{}", if let Some(ct) = self.moment_type.as_ref() { ct.clone() } else {
            return Err(format!("Never called set_moment_type on Clock ({})", self.name).to_string())
        });

        let formatted = rustfmt_wrapper::rustfmt(quote! {
            pub struct #struct_name {}

            impl #struct_name {
                const fn to_moment(rep: #moment_rep) -> ClockMoment<#moment_rep> {
                    ClockMoment::#moment_enum(rep)
                }

                const fn represents() -> &'static str { #repr_name }
            }

            impl ClockLike for #struct_name {
                type MomentRep = #moment_rep;

                fn represents(&self) -> &str { <#struct_name>::represents() }

                fn to_moment(rep: #moment_rep) -> ClockMoment<#moment_rep> {
                    <#struct_name>::to_moment(rep)
                }
            }

            impl AddableClockLike<#moment_rep> for #struct_name {}
        });

        match formatted {
            Ok(formatted_str) => Ok(formatted_str),
            Err(rustfmt_wrapper::Error::Rustfmt(err)) => Err(format!("Error formatting Clock({}):\n{}", self.name, err)),
            Err(err) => Err(format!("Error generating Clock({}):\n{}", self.name, err))
        }
    }
}