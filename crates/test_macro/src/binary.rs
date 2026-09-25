use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

use crate::Expr;

#[derive(Debug)]
pub(crate) struct Binary {
    pub(crate) op: BinaryOp,
    pub(crate) expr: Box<Expr>,
}

impl Parse for Binary {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        let op = fork.parse()?;
        let expr = fork.parse()?;
        input.advance_to(&fork);
        Ok(Binary { op, expr })
    }
}

impl ToTokens for Binary {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Binary { op, expr } = self;
        quote! { #op #expr }.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) enum BinaryOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Rem,
    /// `&&`
    And,
    /// `||`
    Or,
    /// `^`
    BitXor,
    /// `&`
    BitAnd,
    /// `|`
    BitOr,
    /// `<<`
    Shl,
    /// `>>`
    Shr,
    /// `==`
    Eq,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `!=`
    Ne,
    /// `>=`
    Ge,
    /// `>`
    Gt,
    /// `=`
    Assign,
    /// `+=`
    AddAssign,
    /// `-=`
    SubAssign,
    /// `*=`
    MulAssign,
    /// `/=`
    DivAssign,
    /// `%=`
    RemAssign,
    /// `^=`
    BitXorAssign,
    /// `&=`
    BitAndAssign,
    /// `|=`
    BitOrAssign,
    /// `<<=`
    ShlAssign,
    /// `>>=`
    ShrAssign,
}

impl Parse for BinaryOp {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.parse::<Token![<<=]>().is_ok() {
            Ok(BinaryOp::ShlAssign)
        } else if input.parse::<Token![>>=]>().is_ok() {
            Ok(BinaryOp::ShrAssign)
        } else if input.parse::<Token![&&]>().is_ok() {
            Ok(BinaryOp::And)
        } else if input.parse::<Token![||]>().is_ok() {
            Ok(BinaryOp::Or)
        } else if input.parse::<Token![<<]>().is_ok() {
            Ok(BinaryOp::Shl)
        } else if input.parse::<Token![>>]>().is_ok() {
            Ok(BinaryOp::Shr)
        } else if input.parse::<Token![==]>().is_ok() {
            Ok(BinaryOp::Eq)
        } else if input.parse::<Token![<=]>().is_ok() {
            Ok(BinaryOp::Le)
        } else if input.parse::<Token![!=]>().is_ok() {
            Ok(BinaryOp::Ne)
        } else if input.parse::<Token![>=]>().is_ok() {
            Ok(BinaryOp::Ge)
        } else if input.parse::<Token![+=]>().is_ok() {
            Ok(BinaryOp::AddAssign)
        } else if input.parse::<Token![-=]>().is_ok() {
            Ok(BinaryOp::SubAssign)
        } else if input.parse::<Token![*=]>().is_ok() {
            Ok(BinaryOp::MulAssign)
        } else if input.parse::<Token![/=]>().is_ok() {
            Ok(BinaryOp::DivAssign)
        } else if input.parse::<Token![%=]>().is_ok() {
            Ok(BinaryOp::RemAssign)
        } else if input.parse::<Token![^=]>().is_ok() {
            Ok(BinaryOp::BitXorAssign)
        } else if input.parse::<Token![&=]>().is_ok() {
            Ok(BinaryOp::BitAndAssign)
        } else if input.parse::<Token![|=]>().is_ok() {
            Ok(BinaryOp::BitOrAssign)
        } else if input.parse::<Token![+]>().is_ok() {
            Ok(BinaryOp::Add)
        } else if input.parse::<Token![-]>().is_ok() {
            Ok(BinaryOp::Sub)
        } else if input.parse::<Token![*]>().is_ok() {
            Ok(BinaryOp::Mul)
        } else if input.parse::<Token![/]>().is_ok() {
            Ok(BinaryOp::Div)
        } else if input.parse::<Token![%]>().is_ok() {
            Ok(BinaryOp::Rem)
        } else if input.parse::<Token![^]>().is_ok() {
            Ok(BinaryOp::BitXor)
        } else if input.parse::<Token![&]>().is_ok() {
            Ok(BinaryOp::BitAnd)
        } else if input.parse::<Token![|]>().is_ok() {
            Ok(BinaryOp::BitOr)
        } else if input.parse::<Token![<]>().is_ok() {
            Ok(BinaryOp::Lt)
        } else if input.parse::<Token![>]>().is_ok() {
            Ok(BinaryOp::Gt)
        } else {
            input.parse::<Token![=]>()?;
            Ok(BinaryOp::Assign)
        }
    }
}

impl ToTokens for BinaryOp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            BinaryOp::Add => quote! { + },
            BinaryOp::Sub => quote! { - },
            BinaryOp::Mul => quote! { * },
            BinaryOp::Div => quote! { / },
            BinaryOp::Rem => quote! { % },
            BinaryOp::And => quote! { && },
            BinaryOp::Or => quote! { || },
            BinaryOp::BitXor => quote! { ^ },
            BinaryOp::BitAnd => quote! { & },
            BinaryOp::BitOr => quote! { | },
            BinaryOp::Shl => quote! { << },
            BinaryOp::Shr => quote! { >> },
            BinaryOp::Eq => quote! { == },
            BinaryOp::Lt => quote! { < },
            BinaryOp::Le => quote! { <= },
            BinaryOp::Ne => quote! { != },
            BinaryOp::Ge => quote! { >= },
            BinaryOp::Gt => quote! { > },
            BinaryOp::Assign => quote! { = },
            BinaryOp::AddAssign => quote! { += },
            BinaryOp::SubAssign => quote! { -= },
            BinaryOp::MulAssign => quote! { *= },
            BinaryOp::DivAssign => quote! { /= },
            BinaryOp::RemAssign => quote! { %= },
            BinaryOp::BitXorAssign => quote! { ^= },
            BinaryOp::BitAndAssign => quote! { &= },
            BinaryOp::BitOrAssign => quote! { |= },
            BinaryOp::ShlAssign => quote! { <<= },
            BinaryOp::ShrAssign => quote! { >>= },
        }
        .to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::{Binary, binary::BinaryOp, expr::ExprBase};

    #[test]
    fn parse() {
        use crate::assert_parse as a;
        fn a(tokens: TokenStream) -> Binary {
            syn::parse2(tokens).unwrap()
        }

        let binary = a(quote! { + 0123 });
        assert!(matches!(binary.op, BinaryOp::Add));
        assert!(matches!(binary.expr.base, ExprBase::Lit(_)));

        let binary = a(quote! { <<= "foo" });
        assert!(matches!(binary.op, BinaryOp::ShlAssign));
        assert!(matches!(binary.expr.base, ExprBase::Lit(_)));

        a! { Binary, Err(_), < Type> };
    }

    #[test]
    fn parse_op() {
        use crate::assert_parse as a;

        a! { _, Ok(BinaryOp::Add), + };
        a! { _, Ok(BinaryOp::Sub), - };
        a! { _, Ok(BinaryOp::Mul), * };
        a! { _, Ok(BinaryOp::Div), / };
        a! { _, Ok(BinaryOp::Rem), % };
        a! { _, Ok(BinaryOp::And), && };
        a! { _, Ok(BinaryOp::Or), || };
        a! { _, Ok(BinaryOp::BitXor), ^ };
        a! { _, Ok(BinaryOp::BitAnd), & };
        a! { _, Ok(BinaryOp::BitOr), | };
        a! { _, Ok(BinaryOp::Shl), << };
        a! { _, Ok(BinaryOp::Shr), >> };
        a! { _, Ok(BinaryOp::Eq), == };
        a! { _, Ok(BinaryOp::Lt), < };
        a! { _, Ok(BinaryOp::Le), <= };
        a! { _, Ok(BinaryOp::Ne), != };
        a! { _, Ok(BinaryOp::Ge), >= };
        a! { _, Ok(BinaryOp::Gt), > };
        a! { _, Ok(BinaryOp::Assign), = };
        a! { _, Ok(BinaryOp::AddAssign), += };
        a! { _, Ok(BinaryOp::SubAssign), -= };
        a! { _, Ok(BinaryOp::MulAssign), *= };
        a! { _, Ok(BinaryOp::DivAssign), /= };
        a! { _, Ok(BinaryOp::RemAssign), %= };
        a! { _, Ok(BinaryOp::BitXorAssign), ^= };
        a! { _, Ok(BinaryOp::BitAndAssign), &= };
        a! { _, Ok(BinaryOp::BitOrAssign), |= };
        a! { _, Ok(BinaryOp::ShlAssign), <<= };
        a! { _, Ok(BinaryOp::ShrAssign), >>= };

        a! { BinaryOp, Err(_), +- };
        a! { BinaryOp, Err(_), -- };
        a! { BinaryOp, Err(_), *- };
        a! { BinaryOp, Err(_), /- };
        a! { BinaryOp, Err(_), === };
        a! { BinaryOp, Err(_), <<=< };
    }

    #[test]
    fn parse_is_polite() {
        use crate::test::assert_respect;
        assert_respect::<Binary>(quote! { + });
        assert_respect::<Binary>(quote! { <<= });
    }

    #[test]
    fn parse_is_polite_op() {
        use crate::test::assert_respect;
        assert_respect::<BinaryOp>(quote! { ! });
    }

    #[test]
    fn to_tokens() {
        todo!();
        // a(
        //     Binary {
        //         op: BinaryOp::Add,
        //         expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
        //     },
        //     "+ 0123",
        // );
        // a(
        //     Binary {
        //         op: BinaryOp::ShlAssign,
        //         expr: Box::new(expr(ExprBase::Lit(quote! { "foo" }))),
        //     },
        //     "<<= \"foo\"",
        // );
    }

    #[test]
    fn to_tokens_op() {
        use crate::assert_to_tokens as a;
        a! { BinaryOp::Add, + };
        a! { BinaryOp::Sub, - };
        a! { BinaryOp::Mul, * };
        a! { BinaryOp::Div, / };
        a! { BinaryOp::Rem, % };
        a! { BinaryOp::And, && };
        a! { BinaryOp::Or, || };
        a! { BinaryOp::BitXor, ^ };
        a! { BinaryOp::BitAnd, & };
        a! { BinaryOp::BitOr, | };
        a! { BinaryOp::Shl, << };
        a! { BinaryOp::Shr, >> };
        a! { BinaryOp::Eq, == };
        a! { BinaryOp::Lt, < };
        a! { BinaryOp::Le, <= };
        a! { BinaryOp::Ne, != };
        a! { BinaryOp::Ge, >= };
        a! { BinaryOp::Gt, > };
        a! { BinaryOp::Assign, = };
        a! { BinaryOp::AddAssign, += };
        a! { BinaryOp::SubAssign, -= };
        a! { BinaryOp::MulAssign, *= };
        a! { BinaryOp::DivAssign, /= };
        a! { BinaryOp::RemAssign, %= };
        a! { BinaryOp::BitXorAssign, ^= };
        a! { BinaryOp::BitAndAssign, &= };
        a! { BinaryOp::BitOrAssign, |= };
        a! { BinaryOp::ShlAssign, <<= };
        a! { BinaryOp::ShrAssign, >>= };
    }
}
