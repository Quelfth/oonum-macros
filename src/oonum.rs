use {
    self::{branch::branch_oonum, root::root_oonum},
    proc_macro::TokenStream,
    syn::Result,
};

mod branch;
mod common;
mod root;

pub fn oonum(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if args.is_empty() {
        root_oonum(item)
    } else {
        branch_oonum(args, item)
    }
}
