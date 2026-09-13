pub(crate) mod cast;
pub(crate) use cast::Cast;

pub(crate) mod dotted;
pub(crate) use dotted::Dotted;

pub(crate) mod field;
pub(crate) use field::Field;

pub(crate) mod method;
pub(crate) use method::Method;

pub(crate) mod prefix;
pub(crate) use prefix::Prefix;

use proc_macro2::{Ident, Literal, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::{
    Result, Token, bracketed, parenthesized,
    parse::{Parse, ParseStream, discouraged::Speculative},
    parse_macro_input,
    punctuated::Punctuated,
    token::Bracket,
};

#[proc_macro]
pub fn test(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Expr);
    quote! { #input }.into()
}

#[derive(Debug)]
pub(crate) enum Array {
    Values(Vec<Expr>),
    Repeat { value: Box<Expr>, amount: Box<Expr> },
}

impl Parse for Array {
    fn parse(input: ParseStream) -> Result<Self> {
        println!("started parse");
        let fork = input.fork();
        let content;
        bracketed!(content in fork);
        println!("got bracketed {content:?}");

        let fork_1 = content.fork();
        if let Ok(exprs) = Punctuated::<Expr, Token![,]>::parse_terminated(&fork_1) {
            println!("a values");
            input.advance_to(&fork);
            println!("advanced");
            return Ok(Array::Values(exprs.into_iter().collect()));
        }

        let value = content.parse()?;
        _ = content.parse::<Token![;]>()?;
        let amount = content.parse()?;
        input.advance_to(&fork);
        Ok(Array::Repeat { value, amount })
    }
}

impl ToTokens for Array {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Array::Values(values) => quote! { [#(#values),*] },
            Array::Repeat { value, amount } => quote! { [#value; #amount] },
        }
        .to_tokens(tokens);
    }
}

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

#[derive(Debug)]
pub(crate) struct Block {
    pub(crate) kind: BlockKind,
    pub(crate) stmts: Vec<Stmt>,
}

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Block { kind, stmts } = self;
        quote! { #kind { #(#stmts)* } }.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) enum BlockKind {
    Default,
    Async,
    Const,
    Loop,
    Try,
    Unsafe,
}

impl Parse for BlockKind {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.parse::<Token![async]>().is_ok() {
            Ok(BlockKind::Async)
        } else if input.parse::<Token![const]>().is_ok() {
            Ok(BlockKind::Const)
        } else if input.parse::<Token![loop]>().is_ok() {
            Ok(BlockKind::Loop)
        } else if input.parse::<Token![try]>().is_ok() {
            Ok(BlockKind::Try)
        } else if input.parse::<Token![unsafe]>().is_ok() {
            Ok(BlockKind::Unsafe)
        } else {
            Ok(BlockKind::Default)
        }
    }
}

impl ToTokens for BlockKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            BlockKind::Default => {}
            BlockKind::Async => {
                quote! { async }.to_tokens(tokens);
            }
            BlockKind::Const => {
                quote! { const }.to_tokens(tokens);
            }
            BlockKind::Loop => {
                quote! { loop }.to_tokens(tokens);
            }
            BlockKind::Try => {
                quote! { try }.to_tokens(tokens);
            }
            BlockKind::Unsafe => {
                quote! { unsafe }.to_tokens(tokens);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Closure {
    pub(crate) has_async: bool,
    pub(crate) has_move: bool,
    pub(crate) pats: TokenStream,
    pub(crate) expr: Box<Expr>,
}

impl Parse for Closure {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();

        let has_async = fork.parse::<Token![async]>().is_ok();
        let has_move = fork.parse::<Token![move]>().is_ok();

        fork.parse::<Token![|]>()?;
        let mut pats = TokenStream::new();
        while !fork.peek(Token![|]) {
            pats.extend(std::iter::once(fork.parse::<TokenTree>()?));
        }
        fork.parse::<Token![|]>()?;

        let expr = fork.parse()?;

        input.advance_to(&fork);
        Ok(Closure {
            has_async,
            has_move,
            pats,
            expr,
        })
    }
}

impl ToTokens for Closure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Closure {
            has_async,
            has_move,
            pats,
            expr,
        } = self;
        let has_async = has_async.then(|| quote! { async });
        let has_move = has_move.then(|| quote! { move });
        quote! {
            #has_async #has_move |#pats| #expr
        }
        .to_tokens(tokens);
    }
}

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

#[derive(Debug)]
pub(crate) struct Expr {
    pub(crate) attrs: TokenStream,
    pub(crate) prefixes: Vec<Prefix>,
    pub(crate) base: ExprBase,
    pub(crate) suffixes: Vec<Suffix>,
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();

        let mut attrs = TokenStream::new();
        loop {
            let fork_1 = fork.fork();
            if fork_1.parse::<Token![#]>().is_err() {
                break;
            }
            let inner = fork_1.parse::<Token![!]>().ok();

            if !fork_1.peek(Bracket) {
                break;
            }
            let meta;
            bracketed!(meta in fork_1);
            let meta = meta.parse::<TokenStream>().unwrap();

            fork.advance_to(&fork_1);
            quote! { # #inner [#meta] }.to_tokens(&mut attrs);
        }

        let mut prefixes = Vec::new();
        while let Ok(prefix) = fork.parse::<Prefix>() {
            prefixes.push(prefix);
        }

        let base = fork.parse()?;
        input.advance_to(&fork);

        let mut suffixes = Vec::new();
        while let Ok(suffix) = input.parse::<Suffix>() {
            suffixes.push(suffix);
        }

        Ok(Expr {
            attrs,
            prefixes,
            base,
            suffixes,
        })
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Expr {
            attrs,
            prefixes,
            base,
            suffixes,
        } = self;
        quote! {
            #attrs
            #(#prefixes)*
            #base
            #(#suffixes)*
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) enum ExprBase {
    Array(Vec<Expr>),
    Block(Block),
    Closure(Closure),
    ForLoop(),
    Group(Box<Expr>),
    If(),
    Let(),
    Lit(TokenStream),
    Macro(),
    Match(),
    Paren(Box<Expr>),
    Range(),
    Repeat(),
    Struct(),
    Tuple(Vec<Expr>),
    While(),
}

impl Parse for ExprBase {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(lit_parsed) = input.parse::<syn::Lit>() {
            let mut lit = TokenStream::new();
            lit_parsed.to_tokens(&mut lit);
            return Ok(ExprBase::Lit(lit));
        }

        Err(input.error("TODO: error strings"))
    }
}

impl ToTokens for ExprBase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ExprBase::Array(exprs) => quote! { [#(#exprs),*] }.to_tokens(tokens),
            ExprBase::Block(block) => block.to_tokens(tokens),
            ExprBase::Closure(closure) => closure.to_tokens(tokens),
            ExprBase::ForLoop() => todo!(),
            ExprBase::Group(expr) => expr.to_tokens(tokens),
            ExprBase::If() => todo!(),
            ExprBase::Let() => todo!(),
            ExprBase::Lit(lit) => lit.to_tokens(tokens),
            ExprBase::Macro() => todo!(),
            ExprBase::Match() => todo!(),
            ExprBase::Paren(expr) => quote! { (#expr) }.to_tokens(tokens),
            ExprBase::Range() => todo!(),
            ExprBase::Repeat() => todo!(),
            ExprBase::Struct() => todo!(),
            ExprBase::Tuple(exprs) => quote! { (#(#exprs),*) }.to_tokens(tokens),
            ExprBase::While() => todo!(),
        };
    }
}

#[derive(Debug)]
pub(crate) struct Index {
    pub(crate) index: Box<Expr>,
}

impl Parse for Index {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        let content;
        bracketed!(content in fork);
        let index = content.parse()?;
        input.advance_to(&fork);
        Ok(Index { index })
    }
}

impl ToTokens for Index {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Index { index } = self;
        quote! { [#index] }.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) struct Stmt {}

impl ToTokens for Stmt {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        todo!();
    }
}

#[derive(Debug)]
pub(crate) enum Suffix {
    Binary(Binary),
    Call(Call),
    Cast(Cast),
    Dot(Dotted),
    Index(Index),
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
            Suffix::Try => quote! { ? }.to_tokens(tokens),
        };
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Debug;

    use proc_macro2::{TokenStream, TokenTree};
    use quote::{ToTokens, quote};
    use syn::{
        Result,
        parse::{Parse, ParseBuffer, Parser},
        parse_str,
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

    #[track_caller]
    pub(crate) fn assert_to_tokens<T: ToTokens>(lhs: T, rhs: &str) {
        assert_tokens(quote! { #lhs }, parse_str(rhs).unwrap());
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
        let tokens_expected = tokens.into_iter().count();
        assert_eq!(
            tokens_expected, tokens_actual,
            "got an unexpected amount of tokens after parsing",
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use proc_macro2::{TokenStream, TokenTree};
    use quote::{ToTokens, format_ident, quote};
    use syn::{
        Result,
        parse::{Parse, ParseBuffer, Parser},
        parse_str,
    };

    use crate::{
        Array, Binary, BinaryOp, BlockKind, Call, Cast, Closure, Dotted, Expr, ExprBase, Field,
        Index, Method, Prefix, Suffix,
        test::{assert_respect, assert_to_tokens, assert_tokens},
    };

    fn expr(base: ExprBase) -> Expr {
        Expr {
            attrs: TokenStream::new(),
            prefixes: vec![],
            base,
            suffixes: vec![],
        }
    }

    #[test]
    fn array_parse() {
        let Array::Values(values) = parse_str::<Array>("[]").unwrap() else {
            panic!();
        };
        assert!(values.is_empty());

        let Array::Values(values) = parse_str::<Array>("[0123]").unwrap() else {
            panic!();
        };
        assert_eq!(values.len(), 1);
        assert!(matches!(values[0].base, ExprBase::Lit(_)));

        let Array::Values(values) = parse_str::<Array>("[0123,]").unwrap() else {
            panic!();
        };
        assert_eq!(values.len(), 1);
        assert!(matches!(values[0].base, ExprBase::Lit(_)));

        let Array::Values(values) = parse_str::<Array>("[0123, 0123]").unwrap() else {
            panic!();
        };
        assert_eq!(values.len(), 2);
        assert!(matches!(values[0].base, ExprBase::Lit(_)));
        assert!(matches!(values[1].base, ExprBase::Lit(_)));
    }

    #[test]
    fn array_print() {
        assert_to_tokens(Array::Values(vec![]), "[]");
        assert_to_tokens(
            Array::Values(vec![expr(ExprBase::Lit(quote! { 0123 }))]),
            "[0123]",
        );
        assert_to_tokens(
            Array::Values(vec![
                expr(ExprBase::Lit(quote! { 0123 })),
                expr(ExprBase::Lit(quote! { "foo" })),
            ]),
            "[0123, \"foo\"]",
        );

        assert_to_tokens(
            Array::Repeat {
                value: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
                amount: Box::new(expr(ExprBase::Lit(quote! { "foo" }))),
            },
            "[0123; \"foo\"]",
        );
    }

    #[test]
    fn binary_parse() {
        let binary = parse_str::<Binary>("+ 0123").unwrap();
        assert!(matches!(binary.op, BinaryOp::Add));
        assert!(matches!(binary.expr.base, ExprBase::Lit(_)));

        let binary = parse_str::<Binary>("<<= \"foo\"").unwrap();
        assert!(matches!(binary.op, BinaryOp::ShlAssign));
        assert!(matches!(binary.expr.base, ExprBase::Lit(_)));

        _ = parse_str::<Binary>("<Type>").unwrap_err();
    }

    #[test]
    fn binary_print() {
        assert_to_tokens(
            Binary {
                op: BinaryOp::Add,
                expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
            },
            "+ 0123",
        );
        assert_to_tokens(
            Binary {
                op: BinaryOp::ShlAssign,
                expr: Box::new(expr(ExprBase::Lit(quote! { "foo" }))),
            },
            "<<= \"foo\"",
        );
    }

    #[test]
    fn binary_respect() {
        assert_respect::<Binary>(quote! { + });
        assert_respect::<Binary>(quote! { <<= });
    }

    #[test]
    fn binary_op_parse() {
        use BinaryOp::{self as Op, *};

        assert!(matches!(parse_str::<Op>("+").unwrap(), Add));
        assert!(matches!(parse_str::<Op>("-").unwrap(), Sub));
        assert!(matches!(parse_str::<Op>("*").unwrap(), Mul));
        assert!(matches!(parse_str::<Op>("/").unwrap(), Div));
        assert!(matches!(parse_str::<Op>("%").unwrap(), Rem));
        assert!(matches!(parse_str::<Op>("&&").unwrap(), And));
        assert!(matches!(parse_str::<Op>("||").unwrap(), Or));
        assert!(matches!(parse_str::<Op>("^").unwrap(), BitXor));
        assert!(matches!(parse_str::<Op>("&").unwrap(), BitAnd));
        assert!(matches!(parse_str::<Op>("|").unwrap(), BitOr));
        assert!(matches!(parse_str::<Op>("<<").unwrap(), Shl));
        assert!(matches!(parse_str::<Op>(">>").unwrap(), Shr));
        assert!(matches!(parse_str::<Op>("==").unwrap(), Eq));
        assert!(matches!(parse_str::<Op>("<").unwrap(), Lt));
        assert!(matches!(parse_str::<Op>("<=").unwrap(), Le));
        assert!(matches!(parse_str::<Op>("!=").unwrap(), Ne));
        assert!(matches!(parse_str::<Op>(">=").unwrap(), Ge));
        assert!(matches!(parse_str::<Op>(">").unwrap(), Gt));
        assert!(matches!(parse_str::<Op>("=").unwrap(), Assign));
        assert!(matches!(parse_str::<Op>("+=").unwrap(), AddAssign));
        assert!(matches!(parse_str::<Op>("-=").unwrap(), SubAssign));
        assert!(matches!(parse_str::<Op>("*=").unwrap(), MulAssign));
        assert!(matches!(parse_str::<Op>("/=").unwrap(), DivAssign));
        assert!(matches!(parse_str::<Op>("%=").unwrap(), RemAssign));
        assert!(matches!(parse_str::<Op>("^=").unwrap(), BitXorAssign));
        assert!(matches!(parse_str::<Op>("&=").unwrap(), BitAndAssign));
        assert!(matches!(parse_str::<Op>("|=").unwrap(), BitOrAssign));
        assert!(matches!(parse_str::<Op>("<<=").unwrap(), ShlAssign));
        assert!(matches!(parse_str::<Op>(">>=").unwrap(), ShrAssign));

        _ = parse_str::<Op>("+-").unwrap_err();
        _ = parse_str::<Op>("--").unwrap_err();
        _ = parse_str::<Op>("*-").unwrap_err();
        _ = parse_str::<Op>("/-").unwrap_err();
        _ = parse_str::<Op>("===").unwrap_err();
        _ = parse_str::<Op>("<<=<").unwrap_err();
    }

    #[test]
    fn binary_op_print() {
        assert_to_tokens(BinaryOp::Add, "+");
        assert_to_tokens(BinaryOp::Sub, "-");
        assert_to_tokens(BinaryOp::Mul, "*");
        assert_to_tokens(BinaryOp::Div, "/");
        assert_to_tokens(BinaryOp::Rem, "%");
        assert_to_tokens(BinaryOp::And, "&&");
        assert_to_tokens(BinaryOp::Or, "||");
        assert_to_tokens(BinaryOp::BitXor, "^");
        assert_to_tokens(BinaryOp::BitAnd, "&");
        assert_to_tokens(BinaryOp::BitOr, "|");
        assert_to_tokens(BinaryOp::Shl, "<<");
        assert_to_tokens(BinaryOp::Shr, ">>");
        assert_to_tokens(BinaryOp::Eq, "==");
        assert_to_tokens(BinaryOp::Lt, "<");
        assert_to_tokens(BinaryOp::Le, "<=");
        assert_to_tokens(BinaryOp::Ne, "!=");
        assert_to_tokens(BinaryOp::Ge, ">=");
        assert_to_tokens(BinaryOp::Gt, ">");
        assert_to_tokens(BinaryOp::Assign, "=");
        assert_to_tokens(BinaryOp::AddAssign, "+=");
        assert_to_tokens(BinaryOp::SubAssign, "-=");
        assert_to_tokens(BinaryOp::MulAssign, "*=");
        assert_to_tokens(BinaryOp::DivAssign, "/=");
        assert_to_tokens(BinaryOp::RemAssign, "%=");
        assert_to_tokens(BinaryOp::BitXorAssign, "^=");
        assert_to_tokens(BinaryOp::BitAndAssign, "&=");
        assert_to_tokens(BinaryOp::BitOrAssign, "|=");
        assert_to_tokens(BinaryOp::ShlAssign, "<<=");
        assert_to_tokens(BinaryOp::ShrAssign, ">>=");
    }

    #[test]
    fn binary_op_respect() {
        assert_respect::<BinaryOp>(quote! { ! });
    }

    #[test]
    fn block_kind_parse() {
        assert!(matches!(parse_str("async").unwrap(), BlockKind::Async));
        assert!(matches!(parse_str("const").unwrap(), BlockKind::Const));
        assert!(matches!(parse_str("").unwrap(), BlockKind::Default));
        assert!(matches!(parse_str("loop").unwrap(), BlockKind::Loop));
        assert!(matches!(parse_str("try").unwrap(), BlockKind::Try));
        assert!(matches!(parse_str("unsafe").unwrap(), BlockKind::Unsafe));

        _ = parse_str::<BlockKind>("foo").unwrap_err();
    }

    #[test]
    fn block_kind_print() {
        assert_to_tokens(BlockKind::Async, "async");
        assert_to_tokens(BlockKind::Const, "const");
        assert_to_tokens(BlockKind::Default, "");
        assert_to_tokens(BlockKind::Loop, "loop");
        assert_to_tokens(BlockKind::Try, "try");
        assert_to_tokens(BlockKind::Unsafe, "unsafe");
    }

    #[test]
    fn closure_parse() {
        let closure = parse_str::<Closure>("|| 0123").unwrap();
        assert_eq!(closure.has_async, false);
        assert_eq!(closure.has_move, false);
        assert!(closure.pats.is_empty());
        assert!(matches!(closure.expr.base, ExprBase::Lit(_)));

        let closure = parse_str::<Closure>("async || 0123").unwrap();
        assert_eq!(closure.has_async, true);
        assert_eq!(closure.has_move, false);

        let closure = parse_str::<Closure>("move || 0123").unwrap();
        assert_eq!(closure.has_async, false);
        assert_eq!(closure.has_move, true);

        let closure = parse_str::<Closure>("async move || 0123").unwrap();
        assert_eq!(closure.has_async, true);
        assert_eq!(closure.has_move, true);

        let closure = parse_str::<Closure>("|ident: Type| 0123").unwrap();
        assert_tokens(closure.pats, quote! { ident: Type });

        _ = parse_str::<Closure>("move async || 0123").unwrap_err();
    }

    #[test]
    fn closure_print() {
        assert_to_tokens(
            Closure {
                has_async: false,
                has_move: false,
                pats: TokenStream::new(),
                expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
            },
            "|| 0123",
        );
        assert_to_tokens(
            Closure {
                has_async: true,
                has_move: false,
                pats: TokenStream::new(),
                expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
            },
            "async || 0123",
        );
        assert_to_tokens(
            Closure {
                has_async: false,
                has_move: true,
                pats: TokenStream::new(),
                expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
            },
            "move || 0123",
        );
        assert_to_tokens(
            Closure {
                has_async: true,
                has_move: true,
                pats: TokenStream::new(),
                expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
            },
            "async move || 0123",
        );
        assert_to_tokens(
            Closure {
                has_async: false,
                has_move: false,
                pats: quote! { ident: Type },
                expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
            },
            "|ident: Type| 0123",
        );
    }

    #[test]
    fn call_parse() {
        assert!(parse_str::<Call>("()").unwrap().args.is_empty());

        let call = parse_str::<Call>("(0123)").unwrap();
        assert_eq!(call.args.len(), 1);
        assert!(matches!(call.args[0].base, ExprBase::Lit(_)));

        let call = parse_str::<Call>("(\"foo\", \"bar\",)").unwrap();
        assert_eq!(call.args.len(), 2);
        assert!(matches!(call.args[0].base, ExprBase::Lit(_)));
        assert!(matches!(call.args[1].base, ExprBase::Lit(_)));
    }

    #[test]
    fn call_print() {
        assert_to_tokens(Call { args: vec![] }, "()");
        assert_to_tokens(
            Call {
                args: vec![expr(ExprBase::Lit(quote! { 0123 }))],
            },
            "(0123)",
        );
        assert_to_tokens(
            Call {
                args: vec![
                    expr(ExprBase::Lit(quote! { "foo" })),
                    expr(ExprBase::Lit(quote! { "bar" })),
                ],
            },
            "(\"foo\", \"bar\")",
        );
    }

    #[test]
    fn expr_parse() {
        let expr = parse_str::<Expr>("0123").unwrap();
        assert!(expr.attrs.is_empty());
        assert!(expr.prefixes.is_empty());
        assert!(matches!(expr.base, ExprBase::Lit(_)));
        assert!(expr.suffixes.is_empty());

        let expr = parse_str::<Expr>("#[meta] 0123").unwrap();
        assert_tokens(expr.attrs, quote! { #[meta] });
        assert!(expr.prefixes.is_empty());
        assert!(matches!(expr.base, ExprBase::Lit(_)));
        assert!(expr.suffixes.is_empty());

        let expr = parse_str::<Expr>("#[meta(foo = [0123])] 0123").unwrap();
        assert_tokens(expr.attrs, quote! { #[meta(foo = [0123])] });

        let expr = parse_str::<Expr>("#[meta] #[meta_foo] 0123").unwrap();
        assert_tokens(expr.attrs, quote! { #[meta] #[meta_foo] });

        let expr = parse_str::<Expr>("#![meta] 0123").unwrap();
        assert_tokens(expr.attrs, quote! { #![meta] });

        let expr = parse_str::<Expr>("!0123").unwrap();
        assert_eq!(expr.prefixes.len(), 1);
        assert!(matches!(expr.prefixes[0], Prefix::Not));

        let expr = parse_str::<Expr>("!!!0123").unwrap();
        assert_eq!(expr.prefixes.len(), 3);
        assert!(matches!(expr.prefixes[0], Prefix::Not));
        assert!(matches!(expr.prefixes[1], Prefix::Not));
        assert!(matches!(expr.prefixes[2], Prefix::Not));

        let expr = parse_str::<Expr>("0123?").unwrap();
        assert_eq!(expr.suffixes.len(), 1);
        assert!(matches!(expr.suffixes[0], Suffix::Try));

        let expr = parse_str::<Expr>("0123???").unwrap();
        assert_eq!(expr.suffixes.len(), 3);
        assert!(matches!(expr.suffixes[0], Suffix::Try));
        assert!(matches!(expr.suffixes[1], Suffix::Try));
        assert!(matches!(expr.suffixes[2], Suffix::Try));

        let expr = parse_str::<Expr>("#[meta] #[meta] !!!0123???").unwrap();
        assert_tokens(expr.attrs, quote! { #[meta] #[meta] });
        assert_eq!(expr.prefixes.len(), 3);
        assert!(matches!(expr.prefixes[0], Prefix::Not));
        assert!(matches!(expr.prefixes[1], Prefix::Not));
        assert!(matches!(expr.prefixes[2], Prefix::Not));
        assert!(matches!(expr.base, ExprBase::Lit(_)));
        assert_eq!(expr.suffixes.len(), 3);
        assert!(matches!(expr.suffixes[0], Suffix::Try));
        assert!(matches!(expr.suffixes[1], Suffix::Try));
        assert!(matches!(expr.suffixes[2], Suffix::Try));
    }

    #[test]
    fn expr_print() {
        assert_to_tokens(
            Expr {
                attrs: quote! { #![meta] #[meta] },
                prefixes: vec![Prefix::Not, Prefix::Neg],
                base: ExprBase::Lit(quote! { 0123 }),
                suffixes: vec![
                    Suffix::Try,
                    Suffix::Binary(Binary {
                        op: BinaryOp::Add,
                        expr: Box::new(expr(ExprBase::Lit(quote! { "foo" }))),
                    }),
                ],
            },
            "#![meta] #[meta] !-0123? + \"foo\"",
        );
    }

    #[test]
    fn expr_respect() {
        assert_respect::<Expr>(quote! { #[meta] });
        assert_respect::<Expr>(quote! { #[meta] * });
        assert_respect::<Expr>(quote! { #[meta] *&raw & });
    }

    #[test]
    fn index_parse() {
        assert!(matches!(
            parse_str::<Index>("[0123]").unwrap().index.base,
            ExprBase::Lit(_),
        ));
        assert!(matches!(
            parse_str::<Index>("[\"0123\"]").unwrap().index.base,
            ExprBase::Lit(_),
        ));

        _ = parse_str::<Index>("").unwrap_err();
        _ = parse_str::<Index>("[0123 +]").unwrap_err();
    }

    #[test]
    fn index_print() {
        assert_to_tokens(
            Index {
                index: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
            },
            "[0123]",
        );
        assert_to_tokens(
            Index {
                index: Box::new(expr(ExprBase::Lit(quote! { "foo" }))),
            },
            "[\"foo\"]",
        );
    }

    #[test]
    fn stmt_parse() {}

    #[test]
    fn stmt_print() {}

    #[test]
    fn suffix_parse() {
        assert!(matches!(parse_str("+ 0").unwrap(), Suffix::Binary(_)));
        assert!(matches!(parse_str("(0123)").unwrap(), Suffix::Call(_)));
        assert!(matches!(parse_str("as Type").unwrap(), Suffix::Cast(_)));
        assert!(matches!(parse_str(".ident").unwrap(), Suffix::Dot(_)));
        assert!(matches!(parse_str("[0123]").unwrap(), Suffix::Index(_)));
        assert!(matches!(parse_str("?").unwrap(), Suffix::Try));
    }

    #[test]
    fn suffix_print() {
        assert_to_tokens(Suffix::Try, "?");
    }

    #[test]
    fn suffix_respect() {
        assert_respect::<Suffix>(quote! { + });
        assert_respect::<Suffix>(quote! { as });
        assert_respect::<Suffix>(quote! { . });
    }
}
