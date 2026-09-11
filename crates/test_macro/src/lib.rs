use proc_macro2::{Delimiter, Ident, Literal, Spacing, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Index, Member, Result, Token,
    buffer::Cursor,
    parse::{Parse, ParseStream, Parser, StepCursor, discouraged::Speculative},
    parse_macro_input,
    punctuated::Punctuated,
};

#[proc_macro]
pub fn test(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Expr);
    quote! {}.into()
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
    Async,
    Const,
    Default,
    Loop,
    Try,
    Unsafe,
}

impl ToTokens for BlockKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            BlockKind::Async => {
                quote! { async }.to_tokens(tokens);
            }
            BlockKind::Const => {
                quote! { const }.to_tokens(tokens);
            }
            BlockKind::Default => {}
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

        let (has_async, has_move, pats) = fork.step(|step_cursor| {
            let (ident, cursor) = step_cursor.ident().unzip();
            let cursor = cursor.unwrap_or(*step_cursor);
            let has_async = ident.is_some_and(|ident| ident == "async");

            let (ident, cursor) = cursor.ident().unzip();
            let cursor = cursor.unwrap_or(*step_cursor);
            let has_move = ident.is_some_and(|ident| ident == "move");

            let mut cursor = if let Some((punct, cursor)) = cursor.punct()
                && punct.as_char() == '|'
                && let Spacing::Alone = punct.spacing()
            {
                cursor
            } else {
                return Err(step_cursor.error("TODO: error strings"));
            };

            let mut pats = TokenStream::new();
            loop {
                match cursor.token_tree() {
                    Some((TokenTree::Punct(punct), next_cursor))
                        if punct.as_char() == '|'
                            && let Spacing::Alone = punct.spacing() =>
                    {
                        cursor = next_cursor;
                        break;
                    }
                    Some((tt, next_cursor)) => {
                        pats.extend(std::iter::once(tt));
                        cursor = next_cursor;
                    }
                    None => return Err(step_cursor.error("TODO: error string")),
                }
            }

            Ok(((has_async, has_move, pats), cursor))
        })?;

        let expr = Box::new(fork.parse::<Expr>()?);
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
pub(crate) struct Expr {
    pub(crate) attrs: TokenStream,
    pub(crate) prefixes: Vec<ExprPrefix>,
    pub(crate) base: ExprBase,
    pub(crate) suffixes: Vec<ExprSuffix>,
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        todo!();
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
pub(crate) enum ExprPrefix {
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

impl Parse for ExprPrefix {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|step_cursor| {
            let (punct, cursor) = if let Some((punct, cursor)) = step_cursor.punct()
                && let Spacing::Alone = punct.spacing()
            {
                (punct, cursor)
            } else {
                return Err(step_cursor.error("TODO: error strings"));
            };

            match punct.as_char() {
                '*' => (ExprPrefix::Deref, cursor),
                '-' => (ExprPrefix::Neg, cursor),
                '!' => (ExprPrefix::Not, cursor),
                '&' if let Some((ident_1, cursor)) = cursor.ident()
                    && ident_1 == "raw"
                    && let Some((ident_2, cursor)) = cursor.ident()
                    && ident_2 == "const" =>
                {
                    (ExprPrefix::RawConst, cursor)
                }
                '&' if let Some((ident_1, cursor)) = cursor.ident()
                    && ident_1 == "raw"
                    && let Some((ident_2, cursor)) = cursor.ident()
                    && ident_2 == "mut" =>
                {
                    (ExprPrefix::RawMut, cursor)
                }
                '&' if let None = cursor.ident() => (ExprPrefix::Ref, cursor),
                '&' if let Some((ident_1, cursor)) = cursor.ident()
                    && ident_1 == "mut" =>
                {
                    (ExprPrefix::RefMut, cursor)
                }
                _ => return Err(step_cursor.error("TODO: error strings")),
            };

            todo!()
        })
    }
}

impl ToTokens for ExprPrefix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ExprPrefix::Deref => quote! { * },
            ExprPrefix::Neg => quote! { - },
            ExprPrefix::Not => quote! { ! },
            ExprPrefix::RawConst => quote! { &raw },
            ExprPrefix::RawMut => quote! { &raw mut },
            ExprPrefix::Ref => quote! { & },
            ExprPrefix::RefMut => quote! { &mut },
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) enum ExprSuffix {
    Binary(ExprSuffixBinary),
    Call(ExprSuffixCall),
    Cast(ExprSuffixCast),
    Dot(ExprSuffixDot),
    Index(ExprSuffixIndex),
    Try,
}

impl ToTokens for ExprSuffix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ExprSuffix::Binary(binary) => binary.to_tokens(tokens),
            ExprSuffix::Call(call) => call.to_tokens(tokens),
            ExprSuffix::Cast(cast) => cast.to_tokens(tokens),
            ExprSuffix::Dot(dot) => dot.to_tokens(tokens),
            ExprSuffix::Index(index) => index.to_tokens(tokens),
            ExprSuffix::Try => quote! { ? }.to_tokens(tokens),
        };
    }
}

/// `$ExprBinaryKind $Expr`
#[derive(Debug)]
pub(crate) struct ExprSuffixBinary {
    pub(crate) kind: ExprSuffixBinaryKind,
    pub(crate) expr: Box<Expr>,
}

impl ToTokens for ExprSuffixBinary {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ExprSuffixBinary { kind, expr } = self;
        quote! { #kind #expr }.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) enum ExprSuffixBinaryKind {
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

impl Parse for ExprSuffixBinaryKind {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|step_cursor| {
            let (punct, cursor) = match step_cursor.punct() {
                Some((punct_1, cursor)) if let Spacing::Alone = punct_1.spacing() => {
                    (Some((punct_1.as_char(), None)), cursor)
                }
                Some((punct_1, cursor))
                    if let Spacing::Joint = punct_1.spacing()
                        && let Some((punct_2, cursor)) = cursor.punct()
                        && let Spacing::Alone = punct_2.spacing() =>
                {
                    (
                        Some((punct_1.as_char(), Some((punct_2.as_char(), None)))),
                        cursor,
                    )
                }
                Some((punct_1, cursor))
                    if let Spacing::Joint = punct_1.spacing()
                        && let Some((punct_2, cursor)) = cursor.punct()
                        && let Spacing::Joint = punct_2.spacing()
                        && let Some((punct_3, cursor)) = cursor.punct()
                        && let Spacing::Alone = punct_3.spacing() =>
                {
                    (
                        Some((
                            punct_1.as_char(),
                            Some((punct_2.as_char(), Some(punct_3.as_char()))),
                        )),
                        cursor,
                    )
                }
                _ => (None, *step_cursor),
            };

            let kind = match punct {
                Some(('+', None)) => ExprSuffixBinaryKind::Add,
                Some(('-', None)) => ExprSuffixBinaryKind::Sub,
                Some(('*', None)) => ExprSuffixBinaryKind::Mul,
                Some(('/', None)) => ExprSuffixBinaryKind::Div,
                Some(('%', None)) => ExprSuffixBinaryKind::Rem,
                Some(('&', Some(('&', None)))) => ExprSuffixBinaryKind::And,
                Some(('|', Some(('|', None)))) => ExprSuffixBinaryKind::Or,
                Some(('^', None)) => ExprSuffixBinaryKind::BitXor,
                Some(('&', None)) => ExprSuffixBinaryKind::BitAnd,
                Some(('|', None)) => ExprSuffixBinaryKind::BitOr,
                Some(('<', Some(('<', None)))) => ExprSuffixBinaryKind::Shl,
                Some(('>', Some(('>', None)))) => ExprSuffixBinaryKind::Shr,
                Some(('=', Some(('=', None)))) => ExprSuffixBinaryKind::Eq,
                Some(('<', None)) => ExprSuffixBinaryKind::Lt,
                Some(('<', Some(('=', None)))) => ExprSuffixBinaryKind::Le,
                Some(('!', Some(('=', None)))) => ExprSuffixBinaryKind::Ne,
                Some(('>', Some(('=', None)))) => ExprSuffixBinaryKind::Ge,
                Some(('>', None)) => ExprSuffixBinaryKind::Gt,
                Some(('=', None)) => ExprSuffixBinaryKind::Assign,
                Some(('+', Some(('=', None)))) => ExprSuffixBinaryKind::AddAssign,
                Some(('-', Some(('=', None)))) => ExprSuffixBinaryKind::SubAssign,
                Some(('*', Some(('=', None)))) => ExprSuffixBinaryKind::MulAssign,
                Some(('/', Some(('=', None)))) => ExprSuffixBinaryKind::DivAssign,
                Some(('%', Some(('=', None)))) => ExprSuffixBinaryKind::RemAssign,
                Some(('^', Some(('=', None)))) => ExprSuffixBinaryKind::BitXorAssign,
                Some(('&', Some(('=', None)))) => ExprSuffixBinaryKind::BitAndAssign,
                Some(('|', Some(('=', None)))) => ExprSuffixBinaryKind::BitOrAssign,
                Some(('<', Some(('<', Some('='))))) => ExprSuffixBinaryKind::ShlAssign,
                Some(('>', Some(('>', Some('='))))) => ExprSuffixBinaryKind::ShrAssign,
                _ => match input.parse::<syn::BinOp>() {
                    Ok(_) => unreachable!(),
                    Err(error) => return Err(error),
                },
            };
            Ok((kind, cursor))
        })
    }
}

impl ToTokens for ExprSuffixBinaryKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ExprSuffixBinaryKind::Add => quote! { + },
            ExprSuffixBinaryKind::Sub => quote! { - },
            ExprSuffixBinaryKind::Mul => quote! { * },
            ExprSuffixBinaryKind::Div => quote! { / },
            ExprSuffixBinaryKind::Rem => quote! { % },
            ExprSuffixBinaryKind::And => quote! { && },
            ExprSuffixBinaryKind::Or => quote! { || },
            ExprSuffixBinaryKind::BitXor => quote! { ^ },
            ExprSuffixBinaryKind::BitAnd => quote! { & },
            ExprSuffixBinaryKind::BitOr => quote! { | },
            ExprSuffixBinaryKind::Shl => quote! { << },
            ExprSuffixBinaryKind::Shr => quote! { >> },
            ExprSuffixBinaryKind::Eq => quote! { == },
            ExprSuffixBinaryKind::Lt => quote! { < },
            ExprSuffixBinaryKind::Le => quote! { <= },
            ExprSuffixBinaryKind::Ne => quote! { != },
            ExprSuffixBinaryKind::Ge => quote! { >= },
            ExprSuffixBinaryKind::Gt => quote! { > },
            ExprSuffixBinaryKind::Assign => quote! { = },
            ExprSuffixBinaryKind::AddAssign => quote! { += },
            ExprSuffixBinaryKind::SubAssign => quote! { -= },
            ExprSuffixBinaryKind::MulAssign => quote! { *= },
            ExprSuffixBinaryKind::DivAssign => quote! { /= },
            ExprSuffixBinaryKind::RemAssign => quote! { %= },
            ExprSuffixBinaryKind::BitXorAssign => quote! { ^= },
            ExprSuffixBinaryKind::BitAndAssign => quote! { &= },
            ExprSuffixBinaryKind::BitOrAssign => quote! { |= },
            ExprSuffixBinaryKind::ShlAssign => quote! { <<= },
            ExprSuffixBinaryKind::ShrAssign => quote! { >>= },
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) struct ExprSuffixCall {
    pub(crate) args: Vec<Expr>,
}

impl ToTokens for ExprSuffixCall {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ExprSuffixCall { args } = self;
        quote! { ( #(#args),* ) }.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) struct ExprSuffixCast {
    pub(crate) ty: TokenStream,
}

impl ToTokens for ExprSuffixCast {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ExprSuffixCast { ty } = self;
        quote! { as #ty }.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) enum ExprSuffixDot {
    Await,
    Field(ExprSuffixDotField),
    MethodCall(ExprSuffixDotMethodCall),
}

impl Parse for ExprSuffixDot {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![.]>()?;
        if input.parse::<Token![await]>().is_ok() {
            Ok(ExprSuffixDot::Await)
        } else if let Ok(method_call) = input.parse() {
            Ok(ExprSuffixDot::MethodCall(method_call))
        } else if let Ok(field) = input.parse() {
            Ok(ExprSuffixDot::Field(field))
        } else {
            Err(input.error("TODO: error strings"))
        }
    }
}

impl ToTokens for ExprSuffixDot {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ExprSuffixDot::Await => quote! { .await },
            ExprSuffixDot::Field(field) => quote! { . #field },
            ExprSuffixDot::MethodCall(method_call) => quote! { . #method_call },
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug)]
pub(crate) enum ExprSuffixDotField {
    Named(Ident),
    Unnamed(u32),
}

impl Parse for ExprSuffixDotField {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|step_cursor| {
            if let Some((ident, cursor)) = step_cursor.ident() {
                Ok((ExprSuffixDotField::Named(ident), cursor))
            } else if let Some((literal, cursor)) = step_cursor.literal()
                && let string = literal.to_string()
                && string.chars().all(|char| char.is_ascii_digit())
                && let Ok(index) = string.parse()
            {
                Ok((ExprSuffixDotField::Unnamed(index), cursor))
            } else {
                Err(step_cursor.error("TODO: error strings"))
            }
        })
    }
}

impl ToTokens for ExprSuffixDotField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ExprSuffixDotField::Named(ident) => ident.to_tokens(tokens),
            ExprSuffixDotField::Unnamed(index) => Literal::u32_unsuffixed(*index).to_tokens(tokens),
        };
    }
}

#[derive(Debug)]
pub(crate) struct ExprSuffixDotMethodCall {
    pub(crate) ident: Ident,
    pub(crate) turbofish: Option<TokenStream>,
    pub(crate) args: Vec<Expr>,
}

impl Parse for ExprSuffixDotMethodCall {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|step_cursor| {
            let Some((ident, cursor)) = step_cursor.ident() else {
                return Err(step_cursor.error("TODO: error strings"));
            };

            let (turbofish, cursor) = if let Some((punct_1, cursor)) = cursor.punct()
                && punct_1.as_char() == ':'
                && let Spacing::Joint = punct_1.spacing()
                && let Some((punct_2, cursor)) = cursor.punct()
                && punct_2.as_char() == ':'
                && let Spacing::Joint = punct_2.spacing()
                && let Some((punct_3, mut cursor)) = cursor.punct()
                && punct_3.as_char() == '<'
            {
                println!("matched");
                let mut turbofish = TokenStream::new();
                let mut scope = 0;
                loop {
                    let tt = cursor.token_tree();
                    let (tt, next_cursor) = match tt {
                        Some((TokenTree::Punct(punct), next_cursor)) if punct.as_char() == '<' => {
                            scope += 1;
                            (TokenTree::Punct(punct), next_cursor)
                        }
                        Some((TokenTree::Punct(punct), next_cursor))
                            if punct.as_char() == '>' && scope != 0 =>
                        {
                            scope -= 1;
                            (TokenTree::Punct(punct), next_cursor)
                        }
                        Some((TokenTree::Punct(punct), cursor)) if punct.as_char() == '>' => {
                            break (Some(turbofish), cursor);
                        }
                        Some((tt, next_cursor)) => (tt, next_cursor),
                        None => {
                            return Err(step_cursor.error("TODO: error strings"));
                        }
                    };
                    turbofish.extend(std::iter::once(tt));
                    cursor = next_cursor;
                }
            } else {
                (None, cursor)
            };

            let Some((inner_cursor, _, cursor)) = cursor.group(Delimiter::Parenthesis) else {
                return Err(step_cursor.error("TODO: error strings"));
            };

            let mut args = TokenStream::new();
            let Some((mut last_token_tree, mut inner_cursor)) = inner_cursor.token_tree() else {
                return Ok((
                    ExprSuffixDotMethodCall {
                        ident,
                        turbofish,
                        args: vec![],
                    },
                    cursor,
                ));
            };
            while let Some((tt, next_inner_cursor)) = inner_cursor.token_tree() {
                args.extend(std::iter::once(last_token_tree));
                last_token_tree = tt;
                inner_cursor = next_inner_cursor;
            }
            let args = Punctuated::<Expr, Token![,]>::parse_terminated
                .parse2(args)?
                .into_iter()
                .collect();

            Ok((
                ExprSuffixDotMethodCall {
                    ident,
                    turbofish,
                    args,
                },
                cursor,
            ))
        })
    }
}

impl ToTokens for ExprSuffixDotMethodCall {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ExprSuffixDotMethodCall {
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

#[derive(Debug)]
pub(crate) struct ExprSuffixIndex {
    pub(crate) index: Box<Expr>,
}

impl ToTokens for ExprSuffixIndex {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ExprSuffixIndex { index } = self;
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

#[cfg(test)]
mod tests {
    use proc_macro2::{TokenStream, TokenTree};
    use quote::{ToTokens, format_ident, quote};

    use crate::{
        Expr, ExprBase, ExprSuffixDot, ExprSuffixDotField, ExprSuffixDotMethodCall, ExprSuffixIndex,
    };

    #[track_caller]
    fn assert_token_stream(lhs: TokenStream, rhs: TokenStream) {
        let mut lhs = lhs.into_iter();
        let mut rhs = rhs.into_iter();
        while let (lhs, rhs) = (lhs.next(), rhs.next())
            && (lhs.is_some() || rhs.is_some())
        {
            match (&lhs, &rhs) {
                (Some(TokenTree::Group(lhs)), Some(TokenTree::Group(rhs))) => {
                    assert_eq!(lhs.delimiter(), rhs.delimiter());
                    assert_token_stream(lhs.stream(), rhs.stream());
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
    fn assert_to_tokens<T: ToTokens>(lhs: T, rhs: &str) {
        assert_token_stream(quote! { #lhs }, syn::parse_str(rhs).unwrap());
    }

    fn expr(base: ExprBase) -> Box<Expr> {
        Box::new(Expr {
            attrs: TokenStream::new(),
            prefixes: vec![],
            base,
            suffixes: vec![],
        })
    }

    #[test]
    fn parse_dot() {
        let Ok(ExprSuffixDot::Await) = syn::parse_str(".await") else {
            panic!();
        };
        let Ok(ExprSuffixDot::Field(field)) = syn::parse_str(".ident") else {
            panic!();
        };
        let Ok(ExprSuffixDot::MethodCall(method_call)) = syn::parse_str(".call()") else {
            panic!();
        };

        _ = syn::parse_str::<ExprSuffixDot>("await").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDot>(".await()").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDot>(".ident::<>").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDot>(".await::<>()").unwrap_err();
    }

    #[test]
    fn print_dot() {
        assert_to_tokens(ExprSuffixDot::Await, ".await");

        assert_to_tokens(
            ExprSuffixDot::Field(ExprSuffixDotField::Named(format_ident!("ident"))),
            ".ident",
        );
        assert_to_tokens(
            ExprSuffixDot::Field(ExprSuffixDotField::Unnamed(0123)),
            ".123",
        );

        assert_to_tokens(
            ExprSuffixDot::MethodCall(ExprSuffixDotMethodCall {
                ident: format_ident!("ident"),
                turbofish: None,
                args: vec![],
            }),
            ".ident()",
        );
        assert_to_tokens(
            ExprSuffixDot::MethodCall(ExprSuffixDotMethodCall {
                ident: format_ident!("ident"),
                turbofish: Some(TokenStream::new()),
                args: vec![],
            }),
            ".ident::<>()",
        );
        assert_to_tokens(
            ExprSuffixDot::MethodCall(ExprSuffixDotMethodCall {
                ident: format_ident!("ident"),
                turbofish: Some(quote! { Type }),
                args: vec![],
            }),
            ".ident::<Type>()",
        );
    }

    #[test]
    fn parse_dot_field() {
        let Ok(ExprSuffixDotField::Named(ident)) = syn::parse_str("ident") else {
            panic!();
        };
        assert_eq!(ident, "ident");

        let Ok(ExprSuffixDotField::Unnamed(index)) = syn::parse_str("0123") else {
            panic!();
        };
        assert_eq!(index, 123);

        _ = syn::parse_str::<ExprSuffixDotField>("+0").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotField>("0i32").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotField>("!").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotField>("()").unwrap_err();
    }

    #[test]
    fn print_dot_field() {
        assert_to_tokens(ExprSuffixDotField::Named(format_ident!("ident")), "ident");
        assert_to_tokens(ExprSuffixDotField::Unnamed(0123), "123");
    }

    #[test]
    fn parse_dot_method_call() {
        let method_call = syn::parse_str::<ExprSuffixDotMethodCall>("ident()").unwrap();
        assert_eq!(method_call.ident, "ident");
        assert!(method_call.turbofish.is_none());
        assert!(method_call.args.is_empty());

        let method_call = syn::parse_str::<ExprSuffixDotMethodCall>("ident::<>()").unwrap();
        assert_eq!(method_call.ident, "ident");
        assert!(method_call.turbofish.unwrap().is_empty());
        assert!(method_call.args.is_empty());

        let method_call = syn::parse_str::<ExprSuffixDotMethodCall>("ident::<Type>()").unwrap();
        assert_eq!(method_call.ident, "ident");
        assert_token_stream(method_call.turbofish.unwrap(), quote! { Type });
        assert!(method_call.args.is_empty());

        let method_call = syn::parse_str::<ExprSuffixDotMethodCall>("ident::<<Type>>()").unwrap();
        assert_eq!(method_call.ident, "ident");
        assert_token_stream(method_call.turbofish.unwrap(), quote! { <Type> });
        assert!(method_call.args.is_empty());

        _ = syn::parse_str::<ExprSuffixDotMethodCall>("0").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotMethodCall>("!").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotMethodCall>("()").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotMethodCall>("ident").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotMethodCall>("ident:").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotMethodCall>("ident::").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotMethodCall>("ident::<").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotMethodCall>("ident::>").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotMethodCall>("ident::<>").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotMethodCall>("ident::<>(").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotMethodCall>("ident::<<>()").unwrap_err();
        _ = syn::parse_str::<ExprSuffixDotMethodCall>("ident::<>>()").unwrap_err();
    }

    #[test]
    fn print_dot_method_call() {
        assert_to_tokens(
            ExprSuffixDotMethodCall {
                ident: format_ident!("ident"),
                turbofish: None,
                args: vec![],
            },
            "ident()",
        );

        assert_to_tokens(
            ExprSuffixDotMethodCall {
                ident: format_ident!("ident"),
                turbofish: Some(quote! { Type }),
                args: vec![],
            },
            "ident::<Type>()",
        );
    }

    #[test]
    fn print_index() {
        assert_to_tokens(
            ExprSuffixIndex {
                index: expr(ExprBase::Lit(quote! { 0123 })),
            },
            "[0123]",
        );
        assert_to_tokens(
            ExprSuffixIndex {
                index: expr(ExprBase::Lit(quote! { "foo" })),
            },
            "[\"foo\"]",
        );
    }
}
