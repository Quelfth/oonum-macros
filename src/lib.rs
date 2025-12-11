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
