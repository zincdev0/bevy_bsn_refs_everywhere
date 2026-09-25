use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

use crate::Expr;

#[derive(Debug)]
pub(crate) enum RangeSuffix {
    Unbounded,
    Exclusive(Box<Expr>),
    Inclusive(Box<Expr>),
}

impl Parse for RangeSuffix {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        fork.parse::<Token![..]>()?;
        if fork.parse::<Token![=]>().is_ok() {
            let expr = fork.parse()?;
            input.advance_to(&fork);
            Ok(RangeSuffix::Inclusive(expr))
        } else if let Ok(expr) = fork.parse() {
            input.advance_to(&fork);
            Ok(RangeSuffix::Exclusive(expr))
        } else {
            input.advance_to(&fork);
            Ok(RangeSuffix::Unbounded)
        }
    }
}

impl ToTokens for RangeSuffix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            RangeSuffix::Unbounded => quote! { .. }.to_tokens(tokens),
            RangeSuffix::Exclusive(expr) => quote! { ..#expr }.to_tokens(tokens),
            RangeSuffix::Inclusive(expr) => quote! { ..=#expr }.to_tokens(tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use crate::RangeSuffix;

    #[test]
    fn parse() {
        use crate::assert_parse as a;
        a! { _, Ok(RangeSuffix::Unbounded), .. }
        a! { _, Ok(RangeSuffix::Exclusive(_)), ..0123 }
        a! { _, Ok(RangeSuffix::Inclusive(_)), ..=0123 }
    }

    #[test]
    fn parse_is_polite() {
        use crate::test::assert_respect;
        assert_respect::<RangeSuffix>(quote! { . });
        assert_respect::<RangeSuffix>(quote! { ..= });
    }

    #[test]
    fn to_tokens() {
        use crate::assert_to_tokens as a;
        a! { RangeSuffix::Unbounded, .. };
        todo!();
        // a! { RangeSuffix::Inclusive(expr()), .. };
    }
}
