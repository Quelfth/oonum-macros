use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Result, parse::Parse, punctuated::Punctuated, *};

pub struct OonumEnum {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub name: Ident,
    pub variants: Punctuated<Variant, Token![,]>,
}

pub fn validate_enum(r#enum: ItemEnum) -> Result<OonumEnum> {
    let ItemEnum {
        attrs,
        vis,
        ident,
        generics: Generics {
            params,
            where_clause,
            ..
        },
        variants,
        ..
    } = r#enum;
    if !params.is_empty() {
        return Err(Error::new_spanned(
            params,
            "generic parameters are not supported",
        ));
    }
    if where_clause.is_some() {
        return Err(Error::new_spanned(
            where_clause,
            "where clause is not supported",
        ));
    }

    Ok(OonumEnum {
        attrs,
        vis,
        name: ident,
        variants,
    })
}

pub fn collect_variants(
    variants: impl IntoIterator<Item = Variant>,
) -> Result<(Vec<Ident>, Vec<Vec<Attribute>>)> {
    let mut vars = Vec::new();
    let mut var_attrs = Vec::new();

    for Variant {
        attrs,
        ident,
        fields,
        discriminant,
    } in variants
    {
        if !matches!(fields, Fields::Unit) {
            return Err(Error::new_spanned(
                fields,
                "variant fields are not allowed here",
            ));
        }
        if let Some((_, discr)) = discriminant {
            return Err(Error::new_spanned(
                discr,
                "manual discriminants are not supported",
            ));
        }
        vars.push(ident);
        var_attrs.push(attrs);
    }

    Ok((vars, var_attrs))
}

pub fn generate_dispatch(
    attrs: &mut Vec<Attribute>,
    name: &Ident,
    variants: &[Ident],
) -> Result<TokenStream> {
    let mut traits = Vec::new();
    let mut indices = Vec::new();

    for (i, Attribute { style, meta, .. }) in attrs.iter_mut().enumerate() {
        if !matches!(*style, AttrStyle::Outer) {
            continue;
        }
        let Meta::List(MetaList { path, tokens, .. }) = meta else {
            continue;
        };
        let Path {
            leading_colon: None,
            segments,
        } = path
        else {
            continue;
        };
        if segments.len() != 1 {
            continue;
        }
        let PathSegment {
            ident,
            arguments: PathArguments::None,
        } = &segments[0]
        else {
            continue;
        };
        if *ident != "dispatch" {
            continue;
        }

        struct DispatchMeta {
            traits: Vec<Path>,
        }

        impl Parse for DispatchMeta {
            fn parse(input: parse::ParseStream) -> Result<Self> {
                let mut traits = Vec::new();
                if input.peek(Ident) || input.peek(Token![::]) {
                    traits.push(input.parse()?);
                }
                while input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                    if input.peek(Ident) || input.peek(Token![::]) {
                        traits.push(input.parse()?);
                    } else {
                        break;
                    }
                }

                if !input.is_empty() {
                    return Err(Error::new(Span::call_site(), "unexpected tokens in input"));
                }

                Ok(Self { traits })
            }
        }

        let meta = tokens.clone();
        traits.append(&mut parse2::<DispatchMeta>(meta)?.traits);

        indices.push(i);
    }

    for i in indices.into_iter().rev() {
        attrs.remove(i);
    }

    let traits = traits.into_iter();

    let variants = quote! {
        #( #variants, )*
    };

    Ok(quote! {
        #(
            #traits!{
                #name {
                    #variants
                }
            }
        )*
    })
}
