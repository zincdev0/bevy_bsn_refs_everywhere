use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

#[derive(Debug)]
pub(crate) enum Prefix {
    /// `*`
    Deref,
    /// `-`
    Neg,
    /// `!`
    Not,
    /// `&raw const`
    RawConst,
    /// `&raw mut`
    RawMut,
    /// `&`
    Ref,
    /// `&mut`
    RefMut,
}

impl Parse for Prefix {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.parse::<Token![*]>().is_ok() {
            return Ok(Prefix::Deref);
        } else if input.parse::<Token![-]>().is_ok() {
            return Ok(Prefix::Neg);
        } else if input.parse::<Token![!]>().is_ok() {
            return Ok(Prefix::Not);
        }

        input.parse::<Token![&]>()?;
        if let fork = input.fork()
            && fork.parse::<Token![raw]>().is_ok()
        {
            if fork.parse::<Token![const]>().is_ok() {
                input.advance_to(&fork);
                Ok(Prefix::RawConst)
            } else {
                fork.parse::<Token![mut]>()?;
                input.advance_to(&fork);
                Ok(Prefix::RawMut)
            }
        } else if input.parse::<Token![mut]>().is_ok() {
            Ok(Prefix::RefMut)
        } else {
            Ok(Prefix::Ref)
        }
    }
}

impl ToTokens for Prefix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Prefix::Deref => quote! { * },
            Prefix::Neg => quote! { - },
            Prefix::Not => quote! { ! },
            Prefix::RawConst => quote! { &raw const },
            Prefix::RawMut => quote! { &raw mut },
            Prefix::Ref => quote! { & },
            Prefix::RefMut => quote! { &mut },
        }
        .to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use crate::Prefix;

    #[test]
    fn parse() {
        use crate::assert_parse as a;
        a! { _, Ok(Prefix::Deref), * };
        a! { _, Ok(Prefix::Neg), - };
        a! { _, Ok(Prefix::Not), ! };
        a! { _, Ok(Prefix::RawConst), &raw const };
        a! { _, Ok(Prefix::RawMut), &raw mut };
        a! { _, Ok(Prefix::Ref), & };
        a! { _, Ok(Prefix::RefMut), &mut };
        a! { Prefix, Err(_), &raw };
    }

    #[test]
    fn to_tokens() {
        use crate::assert_to_tokens as a;
        a! { Prefix::Deref, * };
        a! { Prefix::Neg, - };
        a! { Prefix::Not, ! };
        a! { Prefix::RawConst, &raw const };
        a! { Prefix::RawMut, &raw mut };
        a! { Prefix::Ref, & };
        a! { Prefix::RefMut, &mut };
    }
}
