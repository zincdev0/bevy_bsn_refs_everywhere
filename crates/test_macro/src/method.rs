use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::{
    Result, Token, parenthesized,
    parse::{Parse, ParseStream, discouraged::Speculative},
    punctuated::Punctuated,
};

use crate::Expr;

#[derive(Debug)]
pub(crate) struct Method {
    pub(crate) ident: Ident,
    pub(crate) turbofish: Option<TokenStream>,
    pub(crate) args: Vec<Expr>,
}

impl Parse for Method {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();

        let ident = fork.parse::<Ident>()?;

        let turbofish = if fork.parse::<Token![::]>().is_ok() {
            fork.parse::<Token![<]>()?;

            let mut turbofish = TokenStream::new();
            let mut scope = 0;
            loop {
                let token_tree = fork.parse::<TokenTree>()?;
                match &token_tree {
                    TokenTree::Punct(punct) if punct.as_char() == '<' => scope += 1,
                    TokenTree::Punct(punct) if punct.as_char() == '>' && scope != 0 => scope -= 1,
                    TokenTree::Punct(punct) if punct.as_char() == '>' => break,
                    _ => {}
                }
                turbofish.extend([token_tree]);
            }

            Some(turbofish)
        } else {
            None
        };

        let args;
        parenthesized!(args in fork);
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(&args)?
            .into_iter()
            .collect();

        input.advance_to(&fork);
        Ok(Method {
            ident,
            turbofish,
            args,
        })
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Method {
            ident,
            turbofish,
            args,
        } = self;
        let turbofish = turbofish
            .as_ref()
            .map(|turbofish| quote! { ::<#turbofish> });
        quote! {
            #ident #turbofish ( #(#args),* )
        }
        .to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};

    use crate::Method;

    #[test]
    fn parse() {
        use crate::{assert_parse as a, test::assert_tokens};
        fn a(tokens: TokenStream) -> Method {
            syn::parse2(tokens).unwrap()
        }

        let method = a(quote! { ident() });
        assert_eq!(method.ident, "ident");
        assert!(method.turbofish.is_none());
        assert!(method.args.is_empty());

        assert_tokens(a(quote! { ident::<>() }).turbofish.unwrap(), quote! {});
        assert_tokens(a(quote! { ident::<T>() }).turbofish.unwrap(), quote! { T });
        assert_tokens(
            a(quote! { ident::<<T>>() }).turbofish.unwrap(),
            quote! { <T> },
        );

        a! { Method, Err(_), 0 };
        a! { Method, Err(_), ! };
        a! { Method, Err(_), () };
        a! { Method, Err(_), ident };
        a! { Method, Err(_), ident: };
        a! { Method, Err(_), ident:: };
        a! { Method, Err(_), ident::< };
        a! { Method, Err(_), ident::> };
        a! { Method, Err(_), ident::<> };
        a! { Method, Err(_), ident::<<>() };
        a! { Method, Err(_), ident::<>>() };
    }

    #[test]
    fn to_tokens() {
        use crate::assert_to_tokens as a;
        a! {
            Method {
                ident: format_ident!("ident"),
                turbofish: None,
                args: vec![],
            },
            ident()
        }
        a! {
            Method {
                ident: format_ident!("ident"),
                turbofish: Some(quote! { Type }),
                args: vec![],
            },
            ident::<Type>()
        }
    }

    #[test]
    fn to_tokens_is_polite() {
        use crate::test::assert_respect;
        assert_respect::<Method>(quote! { method });
    }
}
