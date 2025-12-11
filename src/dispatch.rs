use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

use crate::dispatch::iterator::extract_iter_item;

mod iterator;

pub fn dispatch(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !args.is_empty() {
        return Err(Error::new(Span::call_site(), "arguments are not allowed"));
    }

    let ItemTrait {
        ident: r#trait,
        items,
        ..
    } = parse2(item.clone())?;

    let (sigs, calls, iter_enums) = {
        let mut sigs = Vec::new();
        let mut calls = Vec::new();
        let mut iter_enums = Vec::new();

        for item in items {
            match item {
                TraitItem::Fn(TraitItemFn {
                    sig:
                        Signature {
                            constness,
                            asyncness,
                            unsafety,
                            abi,
                            ident,
                            generics:
                                Generics {
                                    params: generic_params,
                                    where_clause,
                                    ..
                                },
                            inputs,
                            variadic,
                            output,
                            ..
                        },
                    ..
                }) => {
                    if abi.is_some() {
                        return Err(Error::new_spanned(abi, "abi is not allowed"));
                    }
                    if variadic.is_some() {
                        return Err(Error::new_spanned(
                            variadic,
                            "variadic methods are not allowed",
                        ));
                    }

                    let mut args = Vec::new();
                    let mut params = Vec::new();

                    let mut inputs = inputs.into_iter();
                    let receiver = inputs.next();
                    let Some(receiver) = receiver else {
                        return Err(Error::new_spanned(
                            receiver,
                            "dispatch requires all functions to have a `self` parameter",
                        ));
                    };

                    match receiver {
                        FnArg::Receiver(Receiver { self_token, ty, .. }) => {
                            args.push(quote! { value });
                            params.push(quote! { #self_token : #ty })
                        }
                        FnArg::Typed(arg) => {
                            return Err(Error::new_spanned(
                                arg,
                                "dispatch requires all functions to have a `self` parameter",
                            ));
                        }
                    }
                    let mut indices = 0..;

                    for input in inputs {
                        match input {
                            FnArg::Receiver(receiver) => {
                                return Err(Error::new_spanned(
                                    receiver,
                                    "multiple receivers are not allowed",
                                ));
                            }
                            FnArg::Typed(PatType { ty, .. }) => {
                                let name = Ident::new(
                                    &format!("parameter{}", indices.next().unwrap()),
                                    Span::call_site(),
                                );
                                args.push(quote! { #name });
                                params.push(quote! { #name: #ty });
                            }
                        }
                    }

                    let iter_item = extract_iter_item(&output);
                    let iter_enum = iter_item.map(|iter_item| {
                        quote! {
                            enum Iter<$($V,)*> { $( $V($V), )* }
                            impl Iterator for Iter {
                                type Item = #iter_item;

                                fn next(&mut self) -> Option<Self::Item> {
                                    match self {$(
                                        Self::$V(value) => Iterator::next(value),
                                    )*}
                                }
                            }
                        }
                    });

                    sigs.push(quote! {
                        #constness #asyncness #unsafety fn #ident #generic_params (#(#params,)*) #output #where_clause
                    });
                    let mut call = quote! { #r#trait::#ident(#(#args,)*)};
                    if asyncness.is_some() {
                        call = quote! { #call.await }
                    }
                    if unsafety.is_some() {
                        call = quote! { unsafe { #call } }
                    }
                    if iter_enum.is_some() {
                        call = quote! { Iter::$V( #call ) }
                    }

                    calls.push(call);
                    iter_enums.push(iter_enum);
                }
                TraitItem::Macro(item) => {
                    return Err(Error::new_spanned(item, "macros are not allowed here"));
                }
                _ => {
                    return Err(Error::new_spanned(
                        item,
                        "forbidden item in `#[dispatch]` trait, only methods are dispatchable",
                    ));
                }
            }
        }

        (sigs, calls, iter_enums)
    };

    let mut macro_name = r#trait.clone();
    macro_name.set_span(Span::call_site());
    let dunder_macro_name = Ident::new(&format!("__{}", r#trait), Span::call_site());

    Ok(quote! {
        macro_rules! #dunder_macro_name {
            ($E:ty {$($V:ident),* $(,)?}) => {
                impl #r#trait for $E {#(
                    #sigs {
                        #iter_enums
                        match self {$(
                            Self::$V(value) => #calls,
                        )*}
                    }
                )*}
            };
        }
        pub(crate) use #dunder_macro_name as #macro_name;

        #item
    })
}
