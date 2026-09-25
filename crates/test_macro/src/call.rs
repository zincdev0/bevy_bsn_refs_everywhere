use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, Token, parenthesized,
    parse::{Parse, ParseStream, discouraged::Speculative},
    punctuated::Punctuated,
};

use crate::Expr;

#[derive(Debug)]
pub(crate) struct Call {
    pub(crate) args: Vec<Expr>,
}

impl Parse for Call {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        let content;
        parenthesized!(content in fork);
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?
            .into_iter()
            .collect();
        input.advance_to(&fork);
        Ok(Call { args })
    }
}

impl ToTokens for Call {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Call { args } = self;
        quote! { ( #(#args),* ) }.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::{Call, expr::ExprBase};

    #[test]
    fn parse() {
        fn a(tokens: TokenStream) -> Call {
            syn::parse2(tokens).unwrap()
        }

        assert!(a(quote! { () }).args.is_empty());

        let call = a(quote! { (0123) });
        assert_eq!(call.args.len(), 1);
        assert!(matches!(call.args[0].base, ExprBase::Lit(_)));

        let call = a(quote! { ("foo", "bar",) });
        assert_eq!(call.args.len(), 2);
        assert!(matches!(call.args[0].base, ExprBase::Lit(_)));
        assert!(matches!(call.args[1].base, ExprBase::Lit(_)));
    }

    #[test]
    fn parse_is_polite() {
        use crate::test::assert_respect;
        assert_respect::<Call>(quote! { (0123 +) });
    }

    #[test]
    fn to_tokens() {
        use crate::assert_to_tokens as a;
        a! { Call { args: vec![] }, () };
        todo!();
        // a! { Call { args: vec![expr(ExprBase::Lit(quote! { 0123 }))] }, (123) };
        // a! {
        //     Call {
        //         args: vec![
        //             expr(ExprBase::Lit(quote! { "foo" })),
        //             expr(ExprBase::Lit(quote! { "bar" })),
        //         ],
        //     },
        //     ("foo", "bar"),
        // };
    }
}
