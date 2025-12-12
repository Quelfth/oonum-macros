use proc_macro::TokenStream;
use quote::quote;
use syn::*;

use crate::oonum::common::{self, OonumEnum};

pub fn root_oonum(item: TokenStream) -> Result<TokenStream> {
    let OonumEnum {
        mut attrs,
        vis,
        name: root,
        variants,
    } = common::validate_enum(parse(item)?)?;

    let (vars, var_attrs) = common::collect_variants(variants)?;

    let discrs = 0..vars.len() as u16;
    let dispatch = common::generate_dispatch(&mut attrs, &root, &vars)?;

    Ok(quote! {
        #(#attrs)*
        #[repr(u16)]
        #vis enum #root { #(
            #(#var_attrs)*
            #vars(#vars) = <#vars as ::oonum::Discriminant<#root>>::DISCRIMINANT,
        )*}

        #(
            impl ::oonum::Discriminant<#root> for #vars {
                const DISCRIMINANT: u16 = #discrs;
            }

            impl ::oonum::Sub<#root> for #vars {
                fn borrow_super(supe: &#root) -> Option<&Self> {
                    match supe {
                        #root::#vars(value) => Some(value),
                        _ => None,
                    }
                }
                fn borrow_super_mut(supe: &mut #root) -> Option<&mut Self> {
                    match supe {
                        #root::#vars(value) => Some(value),
                        _ => None,
                    }
                }
                fn from_super(supe: #root) -> Option<Self> {
                    match supe {
                        #root::#vars(value) => Some(value),
                        _ => None,
                    }
                }
                fn into_super(self) -> #root {
                    #root::#vars(self)
                }

                fn can_borrow_super(supe: &#root) -> bool {
                    matches!(supe, #root::#vars(_))
                }
            }

            impl From<#vars> for #root {
                fn from(value: #vars) -> Self {
                    value.into_super()
                }
            }
        )*

        #dispatch
    }
    .into())
}
