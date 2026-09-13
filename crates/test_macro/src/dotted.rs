use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

use crate::{Field, Method};

#[derive(Debug)]
pub(crate) enum Dotted {
    Await,
    Field(Field),
    Method(Method),
}

impl Parse for Dotted {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        _ = fork.parse::<Token![.]>()?;
        if let Ok(method_call) = fork.parse() {
            input.advance_to(&fork);
            Ok(Dotted::Method(method_call))
        } else if let Ok(field) = fork.parse() {
            input.advance_to(&fork);
            Ok(Dotted::Field(field))
        } else {
            fork.parse::<Token![await]>()?;
            input.advance_to(&fork);
            Ok(Dotted::Await)
        }
    }
}

impl ToTokens for Dotted {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Dotted::Await => quote! { .await },
            Dotted::Field(field) => quote! { . #field },
            Dotted::Method(method_call) => quote! { . #method_call },
        }
        .to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use quote::{format_ident, quote};

    use crate::{Dotted, Field, Method};

    #[test]
    fn parse() {
        use crate::assert_parse as a;
        a! { _, Ok(Dotted::Await), .await };
        a! { _, Ok(Dotted::Field(_)), .ident };
        a! { _, Ok(Dotted::Method(_)), .call() };
        a! { Dotted, Err(_), await };
        a! { Dotted, Err(_), .await() };
        a! { Dotted, Err(_), .await::<>() };
        a! { Dotted, Err(_), .ident::<> };
    }

    #[test]
    fn to_tokens() {
        use crate::assert_to_tokens as a;
        a!(Dotted::Await, .await);
        a!(Dotted::Field(Field::Named(format_ident!("ident"))), .ident);
        a!(Dotted::Field(Field::Unnamed(0123)), .123);
        a!(Dotted::Method(Method {
            ident: format_ident!("ident"),
            turbofish: None,
            args: vec![],
        }), .ident());
    }

    #[test]
    fn to_tokens_is_polite() {
        use crate::test::assert_respect;
        assert_respect::<Dotted>(quote! { . });
    }
}
