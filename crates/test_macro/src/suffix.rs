use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream},
};

use crate::{Binary, Call, Cast, Dotted, Index, RangeSuffix};

#[derive(Debug)]
pub(crate) enum Suffix {
    Binary(Binary),
    Call(Call),
    Cast(Cast),
    Dot(Dotted),
    Index(Index),
    Range(RangeSuffix),
    Try,
}

impl Parse for Suffix {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(binary) = input.parse() {
            Ok(Suffix::Binary(binary))
        } else if let Ok(call) = input.parse() {
            Ok(Suffix::Call(call))
        } else if let Ok(cast) = input.parse() {
            Ok(Suffix::Cast(cast))
        } else if let Ok(dot) = input.parse() {
            Ok(Suffix::Dot(dot))
        } else if let Ok(index) = input.parse() {
            Ok(Suffix::Index(index))
        } else if let Ok(range) = input.parse() {
            Ok(Suffix::Range(range))
        } else {
            input.parse::<Token![?]>()?;
            Ok(Suffix::Try)
        }
    }
}

impl ToTokens for Suffix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Suffix::Binary(binary) => binary.to_tokens(tokens),
            Suffix::Call(call) => call.to_tokens(tokens),
            Suffix::Cast(cast) => cast.to_tokens(tokens),
            Suffix::Dot(dot) => dot.to_tokens(tokens),
            Suffix::Index(index) => index.to_tokens(tokens),
            Suffix::Range(range) => range.to_tokens(tokens),
            Suffix::Try => quote! { ? }.to_tokens(tokens),
        };
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use crate::Suffix;

    #[test]
    fn parse() {
        use crate::assert_parse as a;
        a! { _, Ok(Suffix::Binary(_)), + 0 };
        a! { _, Ok(Suffix::Call(_)), (0123) };
        a! { _, Ok(Suffix::Cast(_)), as Type };
        a! { _, Ok(Suffix::Dot(_)), .ident };
        a! { _, Ok(Suffix::Index(_)), [0123] };
        a! { _, Ok(Suffix::Range(_)), .. };
        a! { _, Ok(Suffix::Try), ? };
    }

    #[test]
    fn parse_is_polite() {
        use crate::test::assert_respect;
        assert_respect::<Suffix>(quote! { + });
        assert_respect::<Suffix>(quote! { as });
        assert_respect::<Suffix>(quote! { . });
    }

    #[test]
    fn to_tokens() {
        use crate::assert_to_tokens as a;
        a! { Suffix::Try, ? };
    }
}
