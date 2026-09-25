use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

#[derive(Debug)]
pub(crate) struct Cast {
    pub(crate) ty: TokenStream,
}

impl Parse for Cast {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        _ = fork.parse::<Token![as]>()?;
        let ty = fork.parse::<syn::Type>()?;
        input.advance_to(&fork);
        Ok(Cast {
            ty: ty.to_token_stream(),
        })
    }
}

impl ToTokens for Cast {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Cast { ty } = self;
        quote! { as #ty }.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::Cast;

    #[test]
    fn parse() {
        use crate::{assert_parse as a, test::assert_tokens};
        fn a(tokens: TokenStream) -> Cast {
            syn::parse2(tokens).unwrap()
        }

        assert_tokens(a(quote! { as T }).ty, quote! { T });
        assert_tokens(a(quote! { as t }).ty, quote! { t });
        assert_tokens(a(quote! { as a::T }).ty, quote! { a::T });
        assert_tokens(a(quote! { as ::a::T }).ty, quote! { ::a::T });
        assert_tokens(a(quote! { as T<U> }).ty, quote! { T<U> });
        assert_tokens(a(quote! { as T::<U> }).ty, quote! { T::<U> });
        assert_tokens(a(quote! { as <T as U>::V }).ty, quote! { <T as U>::V });
        assert_tokens(a(quote! { as &T }).ty, quote! { &T });
        assert_tokens(a(quote! { as &'a T }).ty, quote! { &'a T });
        assert_tokens(a(quote! { as dyn T }).ty, quote! { dyn T });
        a! { Cast, Err(_), Type };
        a! { Cast, Err(_), <Type> };
        a! { Cast, Err(_), as <Type as Trait> };
    }

    #[test]
    fn parse_is_polite() {
        use crate::test::assert_respect;
        assert_respect::<Cast>(quote! { as });
        assert_respect::<Cast>(quote! { as :: });
        assert_respect::<Cast>(quote! { as module:: });
    }

    #[test]
    fn to_tokens() {
        use crate::assert_to_tokens as a;
        a! { Cast { ty: quote! { Type } }, as Type };
        a! { Cast { ty: quote! { <Type as Trait>::Assoc } }, as <Type as Trait>::Assoc };
    }
}
