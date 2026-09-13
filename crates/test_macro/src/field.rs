use proc_macro2::{Ident, Literal, TokenStream};
use quote::ToTokens;
use syn::{
    Result,
    parse::{Parse, ParseStream},
};

#[derive(Debug)]
pub(crate) enum Field {
    Named(Ident),
    Unnamed(u32),
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        match input.parse::<syn::Member>()? {
            syn::Member::Named(ident) => Ok(Field::Named(ident)),
            syn::Member::Unnamed(index) => Ok(Field::Unnamed(index.index)),
        }
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Field::Named(ident) => ident.to_tokens(tokens),
            Field::Unnamed(index) => Literal::u32_unsuffixed(*index).to_tokens(tokens),
        };
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};

    use crate::Field;

    #[test]
    fn parse() {
        use crate::assert_parse as a;
        fn a(tokens: TokenStream) -> Field {
            syn::parse2(tokens).unwrap()
        }

        match a(quote! { ident }) {
            Field::Named(ident) => assert_eq!(ident, "ident"),
            output => panic!("unexpected output: {output:?}"),
        }
        match a(quote! { 0123 }) {
            Field::Unnamed(index) => assert_eq!(index, 123),
            output => panic!("unexpected output: {output:?}"),
        }

        a! { Field, Err(_), +0 };
        a! { Field, Err(_), 0i32 };
        a! { Field, Err(_), ! };
        a! { Field, Err(_), () };
    }

    #[test]
    fn to_tokens() {
        use crate::assert_to_tokens as a;
        a! { Field::Named(format_ident!("ident")), ident };
        a! { Field::Unnamed(0123), 123 };
    }
}
