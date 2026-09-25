pub(crate) mod array;
pub(crate) use array::Array;

pub(crate) mod binary;
pub(crate) use binary::Binary;

pub(crate) mod block;
pub(crate) use block::{Block, BlockRaw};

pub(crate) mod call;
pub(crate) use call::Call;

pub(crate) mod cast;
pub(crate) use cast::Cast;

pub(crate) mod closure;
pub(crate) use closure::Closure;

pub(crate) mod dotted;
pub(crate) use dotted::Dotted;

pub(crate) mod expr;
pub(crate) use expr::Expr;

pub(crate) mod field;
pub(crate) use field::Field;

pub(crate) mod for_loop;
pub(crate) use for_loop::ForLoop;

pub(crate) mod if_else;
pub(crate) use if_else::IfElse;

pub(crate) mod index;
pub(crate) use index::Index;

pub(crate) mod method;
pub(crate) use method::Method;

pub(crate) mod prefix;
pub(crate) use prefix::Prefix;

pub(crate) mod range_suffix;
pub(crate) use range_suffix::RangeSuffix;

pub(crate) mod stmt;
pub(crate) use stmt::Stmt;

pub(crate) mod suffix;
pub(crate) use suffix::Suffix;

#[proc_macro]
pub fn test(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as Expr);
    quote::quote! { #input }.into()
}

#[cfg(test)]
mod test {
    use std::fmt::Debug;

    use proc_macro2::{TokenStream, TokenTree};
    use syn::{
        Result,
        parse::{Parse, ParseBuffer, Parser},
    };

    #[macro_export]
    macro_rules! assert_parse {
        ($ty:ty, $pat:pat, $($tokens:tt)*) => {
            assert!(matches!(
                ::syn::parse2::<$ty>(::quote::quote! { $($tokens)* }),
                $pat,
            ));
        };
    }

    #[macro_export]
    macro_rules! assert_to_tokens {
        ($value:expr, $($tokens:tt)*) => {
            let value = $value;
            $crate::test::assert_tokens(
                ::quote::quote! { #value },
                ::quote::quote! { $($tokens)* },
            )
        };
    }

    #[track_caller]
    pub(crate) fn assert_tokens(lhs: TokenStream, rhs: TokenStream) {
        let mut lhs = lhs.into_iter();
        let mut rhs = rhs.into_iter();
        while let (lhs, rhs) = (lhs.next(), rhs.next())
            && (lhs.is_some() || rhs.is_some())
        {
            match (&lhs, &rhs) {
                (Some(TokenTree::Group(lhs)), Some(TokenTree::Group(rhs))) => {
                    assert_eq!(lhs.delimiter(), rhs.delimiter());
                    assert_tokens(lhs.stream(), rhs.stream());
                }
                (Some(TokenTree::Ident(lhs)), Some(TokenTree::Ident(rhs))) => {
                    assert_eq!(lhs.to_string(), rhs.to_string());
                }
                (Some(TokenTree::Punct(lhs)), Some(TokenTree::Punct(rhs))) => {
                    assert_eq!(lhs.as_char(), rhs.as_char());
                }
                (Some(TokenTree::Literal(lhs)), Some(TokenTree::Literal(rhs))) => {
                    assert_eq!(lhs.to_string(), rhs.to_string());
                }
                _ => panic!("lhs: {lhs:?}\nrhs: {rhs:?}"),
            }
        }
    }

    /// Asserts that the tokens in [`TokenStream`], which must be a partially valid value of `T` as tokens, are not
    /// consumed when parsing an invalid value of `T`.
    #[track_caller]
    pub(crate) fn assert_respect<T: Parse + Debug>(tokens: TokenStream) {
        fn parse_t_then_count_tokens<T: Parse + Debug>(input: &ParseBuffer) -> Result<usize> {
            _ = input.parse::<T>().unwrap_err();
            Ok(input.parse::<TokenStream>()?.into_iter().count())
        }

        let tokens_actual = parse_t_then_count_tokens::<T>
            .parse2(tokens.clone())
            .unwrap();
        let tokens_expected = tokens.clone().into_iter().count();
        assert_eq!(
            tokens_expected,
            tokens_actual,
            "got an unexpected amount of tokens after parsing:\nparsed tokens: `{}`\nremaining tokens: `{}`",
            tokens
                .clone()
                .into_iter()
                .take(tokens_actual)
                .collect::<TokenStream>(),
            tokens
                .into_iter()
                .skip(tokens_actual)
                .collect::<TokenStream>(),
        );
    }
}
