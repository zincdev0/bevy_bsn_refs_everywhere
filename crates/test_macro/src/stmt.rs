use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(Debug)]
pub(crate) struct Stmt {}

impl ToTokens for Stmt {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        _ = tokens;
        todo!();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse() {}

    #[test]
    fn to_tokens() {}
}
