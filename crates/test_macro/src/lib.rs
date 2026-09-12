use proc_macro2::{Delimiter, Ident, Literal, Spacing, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::{
    Lit, Result, Token,
    parse::{Parse, ParseStream, Parser, discouraged::Speculative},
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
                    Some((token_tree, next_cursor)) => {
                        pats.extend(std::iter::once(token_tree));
                        cursor = next_cursor;
                    }
                    None => return Err(step_cursor.error("TODO: error string")),
                }
            }

            Ok(((has_async, has_move, pats), cursor))
        })?;

        let expr = Box::new(fork.parse()?);
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
        let fork = input.fork();

        let base = fork.parse()?;

        input.advance_to(&fork);
        Ok(Expr {
            attrs: TokenStream::new(),
            prefixes: vec![],
            base,
            suffixes: vec![],
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

impl Parse for ExprSuffixBinary {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind = input.parse()?;
        let expr = Box::new(input.parse()?);
        Ok(ExprSuffixBinary { kind, expr })
    }
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

            use ExprSuffixBinaryKind::*;
            let kind = match punct {
                Some(('+', None)) => Add,
                Some(('-', None)) => Sub,
                Some(('*', None)) => Mul,
                Some(('/', None)) => Div,
                Some(('%', None)) => Rem,
                Some(('&', Some(('&', None)))) => And,
                Some(('|', Some(('|', None)))) => Or,
                Some(('^', None)) => BitXor,
                Some(('&', None)) => BitAnd,
                Some(('|', None)) => BitOr,
                Some(('<', Some(('<', None)))) => Shl,
                Some(('>', Some(('>', None)))) => Shr,
                Some(('=', Some(('=', None)))) => Eq,
                Some(('<', None)) => Lt,
                Some(('<', Some(('=', None)))) => Le,
                Some(('!', Some(('=', None)))) => Ne,
                Some(('>', Some(('=', None)))) => Ge,
                Some(('>', None)) => Gt,
                Some(('=', None)) => Assign,
                Some(('+', Some(('=', None)))) => AddAssign,
                Some(('-', Some(('=', None)))) => SubAssign,
                Some(('*', Some(('=', None)))) => MulAssign,
                Some(('/', Some(('=', None)))) => DivAssign,
                Some(('%', Some(('=', None)))) => RemAssign,
                Some(('^', Some(('=', None)))) => BitXorAssign,
                Some(('&', Some(('=', None)))) => BitAndAssign,
                Some(('|', Some(('=', None)))) => BitOrAssign,
                Some(('<', Some(('<', Some('='))))) => ShlAssign,
                Some(('>', Some(('>', Some('='))))) => ShrAssign,
                _ => return Err(step_cursor.error("TODO: error strings")),
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

impl Parse for ExprSuffixCast {
    fn parse(input: ParseStream) -> Result<Self> {
        // using `syn::Type` to fully parse the type instead of taking shortcuts, because I don't think casts would be
        // extremely common and types without surrounding `<>` are very complex to skip.
        //
        // if this needs to be changed later, things to look out for are:
        // - `module::T`
        // - `::module::T`
        // - `<T as U>::Assoc`
        // - `&T`
        // - `&'a T`
        // - `dyn T`
        _ = input.parse::<Token![as]>()?;
        let ty_parsed = input.parse::<syn::Type>()?;
        let mut ty = TokenStream::new();
        ty_parsed.to_tokens(&mut ty);
        Ok(ExprSuffixCast { ty })
    }
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
        _ = input.parse::<Token![.]>()?;
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
                    let token_tree = cursor.token_tree();
                    let (token_tree, next_cursor) = match token_tree {
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
                        Some((token_tree, next_cursor)) => (token_tree, next_cursor),
                        None => {
                            return Err(step_cursor.error("TODO: error strings"));
                        }
                    };
                    turbofish.extend(std::iter::once(token_tree));
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
            while let Some((token_tree, next_inner_cursor)) = inner_cursor.token_tree() {
                args.extend(std::iter::once(last_token_tree));
                last_token_tree = token_tree;
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
    use syn::parse_str;

    use crate::{
        Expr, ExprBase, ExprSuffixBinary, ExprSuffixBinaryKind, ExprSuffixCast, ExprSuffixDot,
        ExprSuffixDotField, ExprSuffixDotMethodCall, ExprSuffixIndex,
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
        assert_token_stream(quote! { #lhs }, parse_str(rhs).unwrap());
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
    fn parse_binary() {
        let binary = parse_str::<ExprSuffixBinary>("+ 0123").unwrap();
        assert!(matches!(binary.kind, ExprSuffixBinaryKind::Add));
        assert!(matches!(binary.expr.base, ExprBase::Lit(_)));

        let binary = parse_str::<ExprSuffixBinary>("<<= \"foo\"").unwrap();
        assert!(matches!(binary.kind, ExprSuffixBinaryKind::ShlAssign));
        assert!(matches!(binary.expr.base, ExprBase::Lit(_)));
    }

    #[test]
    fn print_binary() {
        assert_to_tokens(
            ExprSuffixBinary {
                kind: ExprSuffixBinaryKind::Add,
                expr: expr(ExprBase::Lit(quote! { 0123 })),
            },
            "+ 0123",
        );
        assert_to_tokens(
            ExprSuffixBinary {
                kind: ExprSuffixBinaryKind::ShlAssign,
                expr: expr(ExprBase::Lit(quote! { "foo" })),
            },
            "<<= \"foo\"",
        );
    }

    #[test]
    fn parse_binary_kind() {
        use ExprSuffixBinaryKind::{self as Kind, *};

        assert!(matches!(parse_str::<Kind>("+").unwrap(), Add));
        assert!(matches!(parse_str::<Kind>("-").unwrap(), Sub));
        assert!(matches!(parse_str::<Kind>("*").unwrap(), Mul));
        assert!(matches!(parse_str::<Kind>("/").unwrap(), Div));
        assert!(matches!(parse_str::<Kind>("%").unwrap(), Rem));
        assert!(matches!(parse_str::<Kind>("&&").unwrap(), And));
        assert!(matches!(parse_str::<Kind>("||").unwrap(), Or));
        assert!(matches!(parse_str::<Kind>("^").unwrap(), BitXor));
        assert!(matches!(parse_str::<Kind>("&").unwrap(), BitAnd));
        assert!(matches!(parse_str::<Kind>("|").unwrap(), BitOr));
        assert!(matches!(parse_str::<Kind>("<<").unwrap(), Shl));
        assert!(matches!(parse_str::<Kind>(">>").unwrap(), Shr));
        assert!(matches!(parse_str::<Kind>("==").unwrap(), Eq));
        assert!(matches!(parse_str::<Kind>("<").unwrap(), Lt));
        assert!(matches!(parse_str::<Kind>("<=").unwrap(), Le));
        assert!(matches!(parse_str::<Kind>("!=").unwrap(), Ne));
        assert!(matches!(parse_str::<Kind>(">=").unwrap(), Ge));
        assert!(matches!(parse_str::<Kind>(">").unwrap(), Gt));
        assert!(matches!(parse_str::<Kind>("=").unwrap(), Assign));
        assert!(matches!(parse_str::<Kind>("+=").unwrap(), AddAssign));
        assert!(matches!(parse_str::<Kind>("-=").unwrap(), SubAssign));
        assert!(matches!(parse_str::<Kind>("*=").unwrap(), MulAssign));
        assert!(matches!(parse_str::<Kind>("/=").unwrap(), DivAssign));
        assert!(matches!(parse_str::<Kind>("%=").unwrap(), RemAssign));
        assert!(matches!(parse_str::<Kind>("^=").unwrap(), BitXorAssign));
        assert!(matches!(parse_str::<Kind>("&=").unwrap(), BitAndAssign));
        assert!(matches!(parse_str::<Kind>("|=").unwrap(), BitOrAssign));
        assert!(matches!(parse_str::<Kind>("<<=").unwrap(), ShlAssign));
        assert!(matches!(parse_str::<Kind>(">>=").unwrap(), ShrAssign));

        _ = parse_str::<Kind>("+-").unwrap_err();
        _ = parse_str::<Kind>("--").unwrap_err();
        _ = parse_str::<Kind>("*-").unwrap_err();
        _ = parse_str::<Kind>("/-").unwrap_err();
        _ = parse_str::<Kind>("===").unwrap_err();
        _ = parse_str::<Kind>("<<=<").unwrap_err();
    }

    #[test]
    fn print_binary_kind() {
        assert_to_tokens(ExprSuffixBinaryKind::Add, "+");
        assert_to_tokens(ExprSuffixBinaryKind::Sub, "-");
        assert_to_tokens(ExprSuffixBinaryKind::Mul, "*");
        assert_to_tokens(ExprSuffixBinaryKind::Div, "/");
        assert_to_tokens(ExprSuffixBinaryKind::Rem, "%");
        assert_to_tokens(ExprSuffixBinaryKind::And, "&&");
        assert_to_tokens(ExprSuffixBinaryKind::Or, "||");
        assert_to_tokens(ExprSuffixBinaryKind::BitXor, "^");
        assert_to_tokens(ExprSuffixBinaryKind::BitAnd, "&");
        assert_to_tokens(ExprSuffixBinaryKind::BitOr, "|");
        assert_to_tokens(ExprSuffixBinaryKind::Shl, "<<");
        assert_to_tokens(ExprSuffixBinaryKind::Shr, ">>");
        assert_to_tokens(ExprSuffixBinaryKind::Eq, "==");
        assert_to_tokens(ExprSuffixBinaryKind::Lt, "<");
        assert_to_tokens(ExprSuffixBinaryKind::Le, "<=");
        assert_to_tokens(ExprSuffixBinaryKind::Ne, "!=");
        assert_to_tokens(ExprSuffixBinaryKind::Ge, ">=");
        assert_to_tokens(ExprSuffixBinaryKind::Gt, ">");
        assert_to_tokens(ExprSuffixBinaryKind::Assign, "=");
        assert_to_tokens(ExprSuffixBinaryKind::AddAssign, "+=");
        assert_to_tokens(ExprSuffixBinaryKind::SubAssign, "-=");
        assert_to_tokens(ExprSuffixBinaryKind::MulAssign, "*=");
        assert_to_tokens(ExprSuffixBinaryKind::DivAssign, "/=");
        assert_to_tokens(ExprSuffixBinaryKind::RemAssign, "%=");
        assert_to_tokens(ExprSuffixBinaryKind::BitXorAssign, "^=");
        assert_to_tokens(ExprSuffixBinaryKind::BitAndAssign, "&=");
        assert_to_tokens(ExprSuffixBinaryKind::BitOrAssign, "|=");
        assert_to_tokens(ExprSuffixBinaryKind::ShlAssign, "<<=");
        assert_to_tokens(ExprSuffixBinaryKind::ShrAssign, ">>=");
    }

    #[test]
    fn parse_cast() {
        let cast = parse_str::<ExprSuffixCast>("as Type").unwrap();
        assert_token_stream(cast.ty, quote! { Type });

        let cast = parse_str::<ExprSuffixCast>("as module::Type").unwrap();
        assert_token_stream(cast.ty, quote! { module::Type });

        let cast = parse_str::<ExprSuffixCast>("as ::module::Type").unwrap();
        assert_token_stream(cast.ty, quote! { ::module::Type });

        let cast = parse_str::<ExprSuffixCast>("as ::module::Type<T>").unwrap();
        assert_token_stream(cast.ty, quote! { ::module::Type<T> });

        let cast = parse_str::<ExprSuffixCast>("as ::module::Type::<T>").unwrap();
        assert_token_stream(cast.ty, quote! { ::module::Type::<T> });

        let cast = parse_str::<ExprSuffixCast>("as <Type as Trait>::Assoc").unwrap();
        assert_token_stream(cast.ty, quote! { <Type as Trait>::Assoc });

        let cast = parse_str::<ExprSuffixCast>("as &T").unwrap();
        assert_token_stream(cast.ty, quote! { &T });

        let cast = parse_str::<ExprSuffixCast>("as &'a T").unwrap();
        assert_token_stream(cast.ty, quote! { &'a T });

        let cast = parse_str::<ExprSuffixCast>("as dyn T").unwrap();
        assert_token_stream(cast.ty, quote! { dyn T });
    }

    #[test]
    fn print_cast() {
        assert_to_tokens(
            ExprSuffixCast {
                ty: quote! { Type },
            },
            "as Type",
        );
        assert_to_tokens(
            ExprSuffixCast {
                ty: quote! { <Type as Trait>::Assoc },
            },
            "as <Type as Trait>::Assoc",
        );
    }

    #[test]
    fn parse_dot() {
        let Ok(ExprSuffixDot::Await) = parse_str(".await") else {
            panic!();
        };
        let Ok(ExprSuffixDot::Field(field)) = parse_str(".ident") else {
            panic!();
        };
        let Ok(ExprSuffixDot::MethodCall(method_call)) = parse_str(".call()") else {
            panic!();
        };

        _ = parse_str::<ExprSuffixDot>("await").unwrap_err();
        _ = parse_str::<ExprSuffixDot>(".await()").unwrap_err();
        _ = parse_str::<ExprSuffixDot>(".ident::<>").unwrap_err();
        _ = parse_str::<ExprSuffixDot>(".await::<>()").unwrap_err();
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
        let Ok(ExprSuffixDotField::Named(ident)) = parse_str("ident") else {
            panic!();
        };
        assert_eq!(ident, "ident");

        let Ok(ExprSuffixDotField::Unnamed(index)) = parse_str("0123") else {
            panic!();
        };
        assert_eq!(index, 123);

        _ = parse_str::<ExprSuffixDotField>("+0").unwrap_err();
        _ = parse_str::<ExprSuffixDotField>("0i32").unwrap_err();
        _ = parse_str::<ExprSuffixDotField>("!").unwrap_err();
        _ = parse_str::<ExprSuffixDotField>("()").unwrap_err();
    }

    #[test]
    fn print_dot_field() {
        assert_to_tokens(ExprSuffixDotField::Named(format_ident!("ident")), "ident");
        assert_to_tokens(ExprSuffixDotField::Unnamed(0123), "123");
    }

    #[test]
    fn parse_dot_method_call() {
        let method_call = parse_str::<ExprSuffixDotMethodCall>("ident()").unwrap();
        assert_eq!(method_call.ident, "ident");
        assert!(method_call.turbofish.is_none());
        assert!(method_call.args.is_empty());

        let method_call = parse_str::<ExprSuffixDotMethodCall>("ident::<>()").unwrap();
        assert_eq!(method_call.ident, "ident");
        assert!(method_call.turbofish.unwrap().is_empty());
        assert!(method_call.args.is_empty());

        let method_call = parse_str::<ExprSuffixDotMethodCall>("ident::<Type>()").unwrap();
        assert_eq!(method_call.ident, "ident");
        assert_token_stream(method_call.turbofish.unwrap(), quote! { Type });
        assert!(method_call.args.is_empty());

        let method_call = parse_str::<ExprSuffixDotMethodCall>("ident::<<Type>>()").unwrap();
        assert_eq!(method_call.ident, "ident");
        assert_token_stream(method_call.turbofish.unwrap(), quote! { <Type> });
        assert!(method_call.args.is_empty());

        _ = parse_str::<ExprSuffixDotMethodCall>("0").unwrap_err();
        _ = parse_str::<ExprSuffixDotMethodCall>("!").unwrap_err();
        _ = parse_str::<ExprSuffixDotMethodCall>("()").unwrap_err();
        _ = parse_str::<ExprSuffixDotMethodCall>("ident").unwrap_err();
        _ = parse_str::<ExprSuffixDotMethodCall>("ident:").unwrap_err();
        _ = parse_str::<ExprSuffixDotMethodCall>("ident::").unwrap_err();
        _ = parse_str::<ExprSuffixDotMethodCall>("ident::<").unwrap_err();
        _ = parse_str::<ExprSuffixDotMethodCall>("ident::>").unwrap_err();
        _ = parse_str::<ExprSuffixDotMethodCall>("ident::<>").unwrap_err();
        _ = parse_str::<ExprSuffixDotMethodCall>("ident::<>(").unwrap_err();
        _ = parse_str::<ExprSuffixDotMethodCall>("ident::<<>()").unwrap_err();
        _ = parse_str::<ExprSuffixDotMethodCall>("ident::<>>()").unwrap_err();
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
