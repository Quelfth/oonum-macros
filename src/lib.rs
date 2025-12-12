use proc_macro::TokenStream;

mod dispatch;
mod oonum;

#[proc_macro_attribute]
pub fn oonum(args: TokenStream, item: TokenStream) -> TokenStream {
    oonum::oonum(args, item).unwrap_or_else(|e| e.to_compile_error().into())
}

#[proc_macro_attribute]
pub fn dispatch(args: TokenStream, item: TokenStream) -> TokenStream {
    dispatch::dispatch(args.into(), item.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[cfg(feature = "function-style")]
#[proc_macro]
pub fn dispatch_(item: TokenStream) -> TokenStream {
    dispatch::dispatch(TokenStream::new().into(), item.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[cfg(feature = "function-style")]
#[proc_macro]
pub fn oonum_(args: TokenStream) -> TokenStream {
    use proc_macro::{Punct, TokenTree};
    use quote::quote;
    let mut iter = args.into_iter().peekable();
    if let Some(TokenTree::Punct(punct)) = iter.peek()
        && punct.as_char() == '@'
    {
        iter.next();
        let Some(TokenTree::Group(group)) = iter.next() else {
            return quote! { compile_error!("@ must be followed by ( )") }.into();
        };

        return oonum::oonum(group.stream(), iter.collect())
            .unwrap_or_else(|e| e.to_compile_error().into());
    }

    oonum::oonum(TokenStream::new(), iter.collect()).unwrap_or_else(|e| e.to_compile_error().into())
}
