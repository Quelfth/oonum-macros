use proc_macro::TokenStream;
use quote::quote;
use syn::*;

use crate::oonum::common::{self, OonumEnum};

pub fn branch_oonum(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let root: Ident = syn::parse(args)?;

    let OonumEnum {
        mut attrs,
        vis,
        name: branch,
        variants,
    } = common::validate_enum(parse(item)?)?;

    let (vars, var_attrs) = common::collect_variants(variants)?;
    let dispatch = common::generate_dispatch(&mut attrs, &branch, &vars)?;

    Ok(quote! {
        #(#attrs)*
        #[repr(u16)]
        #vis enum #branch {#(
            #(#var_attrs)*
            #vars(#vars) = <#vars as ::oonum::Discriminant<#root>>::DISCRIMINANT,
        )*}

        impl ::oonum::Sub<#root> for #branch {
            fn borrow_super(supe: &#root) -> Option<&Self> {
                match supe {
                    #(#root::#vars(_) => true,)*
                    _ => false
                }
                .then(|| unsafe { std::mem::transmute(supe) })
            }
            fn borrow_super_mut(supe: &mut #root) -> Option<&mut Self> {
                match supe {
                    #(#root::#vars(_) => true,)*
                    _ => false
                }
                .then(|| unsafe { std::mem::transmute(supe) })
            }
            fn from_super(supe: #root) -> Option<Self> {
                match supe {
                    #(#root::#vars(value) => Some(Self::#vars(value)),)*
                    _ => None,
                }
            }
            fn into_super(self) -> #root {
                match self {
                    #(Self::#vars(value) => #root::#vars(value),)*
                }
            }

            fn can_borrow_super(supe: &#root) -> bool {
                match supe {
                    #(#root::#vars(_) => true,)*
                    _ => false
                }
            }


        }

        #(
            impl ::oonum::Sub<#branch> for #vars {
                fn borrow_super(supe: &#branch) -> Option<&Self> {
                    match supe {
                        #branch::#vars(value) => Some(value),
                        _ => None,
                    }
                }
                fn borrow_super_mut(supe: &mut #branch) -> Option<&mut Self> {
                    match supe {
                        #branch::#vars(value) => Some(value),
                        _ => None,
                    }
                }
                fn from_super(supe: #branch) -> Option<Self> {
                    match supe {
                        #branch::#vars(value) => Some(value),
                        _ => None,
                    }
                }
                fn into_super(self) -> #branch {
                    #branch::#vars(self)
                }

                fn can_borrow_super(supe: &#branch) -> bool {
                    matches!(supe, #branch::#vars(_))
                }
            }

            impl From<#vars> for #branch {
                fn from(value: #vars) -> Self {
                    value.into_super()
                }
            }

        )*

        impl From<#branch> for #root {
            fn from(value: #branch) -> Self {
                value.into_super()
            }
        }

        #dispatch
    }
    .into())
}
