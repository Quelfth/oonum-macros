use syn::*;

pub fn extract_iter_item(r#type: &ReturnType) -> Option<Type> {
    if let ReturnType::Type(_, r#type) = r#type
        && let Type::ImplTrait(TypeImplTrait { bounds, .. }) = &**r#type
    {
        bounds
            .iter()
            .filter_map(|b| match b {
                TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path,
                }) => iter_item_from_path(path),
                _ => None,
            })
            .next()
    } else {
        None
    }
}

fn iter_item_from_path(path: &Path) -> Option<Type> {
    let Path {
        leading_colon: None,
        segments,
    } = path
    else {
        return None;
    };
    if segments.len() != 1 {
        return None;
    }

    let PathSegment { ident, arguments } = &segments[0];
    if *ident != "Iterator" {
        return None;
    }

    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = arguments
    else {
        return None;
    };
    if args.len() != 1 {
        return None;
    }
    let GenericArgument::AssocType(AssocType { ident, ty, .. }) = &args[0] else {
        return None;
    };
    if *ident != "Item" {
        return None;
    }

    Some(ty.clone())
}
