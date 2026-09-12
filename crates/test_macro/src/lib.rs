use proc_macro2::{Delimiter, Ident, Literal, Spacing, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::{
    Result, Token, bracketed, parenthesized,
    parse::{Parse, ParseStream, Parser, discouraged::Speculative},
    parse_macro_input,
    punctuated::Punctuated,
};

#[proc_macro]
pub fn test(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Expr);
    quote! { #input }.into()
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
pub(crate) struct Expr {
    pub(crate) attrs: TokenStream,
    pub(crate) prefixes: Vec<ExprPrefix>,
    pub(crate) base: ExprBase,
    pub(crate) suffixes: Vec<ExprSuffix>,
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();

        let attrs = fork.step(|step_cursor| {
            let mut attrs = TokenStream::new();
            let mut cursor = *step_cursor;

            loop {
                if let Some((punct_1, next_cursor)) = cursor.punct()
                    && punct_1.as_char() == '#'
                {
                    cursor = next_cursor;
                } else {
                    break Ok((attrs, cursor));
                }

                let is_inner = if let Some((punct_2, next_cursor)) = cursor.punct()
                    && punct_2.as_char() == '!'
                {
                    cursor = next_cursor;
                    true
                } else {
                    false
                };

                let Some((inner_cursor, _, next_cursor)) = cursor.group(Delimiter::Bracket) else {
                    break Ok((attrs, cursor));
                };

                let is_inner = is_inner.then(|| quote! { ! });
                let inner_cursor = inner_cursor.token_stream();
                quote! { # #is_inner [#inner_cursor] }.to_tokens(&mut attrs);
                cursor = next_cursor;
            }
        })?;

        let mut prefixes = Vec::new();
        while let Ok(prefix) = fork.parse::<ExprPrefix>() {
            prefixes.push(prefix);
        }

        let base = fork.parse()?;
        input.advance_to(&fork);

        let mut suffixes = Vec::new();
        while let Ok(suffix) = input.parse::<ExprSuffix>() {
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
            let (punct, cursor) = if let Some((punct, cursor)) = step_cursor.punct() {
                (punct, cursor)
            } else {
                return Err(step_cursor.error("TODO: error strings"));
            };

            match punct.as_char() {
                '*' => Ok((ExprPrefix::Deref, cursor)),
                '-' => Ok((ExprPrefix::Neg, cursor)),
                '!' => Ok((ExprPrefix::Not, cursor)),
                '&' if let Some((ident_1, cursor)) = cursor.ident()
                    && ident_1 == "raw"
                    && let Some((ident_2, cursor)) = cursor.ident()
                    && ident_2 == "const" =>
                {
                    Ok((ExprPrefix::RawConst, cursor))
                }
                '&' if let Some((ident_1, cursor)) = cursor.ident()
                    && ident_1 == "raw"
                    && let Some((ident_2, cursor)) = cursor.ident()
                    && ident_2 == "mut" =>
                {
                    Ok((ExprPrefix::RawMut, cursor))
                }
                '&' if let None = cursor.ident() => Ok((ExprPrefix::Ref, cursor)),
                '&' if let Some((ident_1, cursor)) = cursor.ident()
                    && ident_1 == "mut" =>
                {
                    Ok((ExprPrefix::RefMut, cursor))
                }
                _ => Err(step_cursor.error("TODO: error strings")),
            }
        })
    }
}

impl ToTokens for ExprPrefix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ExprPrefix::Deref => quote! { * },
            ExprPrefix::Neg => quote! { - },
            ExprPrefix::Not => quote! { ! },
            ExprPrefix::RawConst => quote! { &raw const },
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

impl Parse for ExprSuffix {
    fn parse(input: ParseStream) -> Result<Self> {
        if let fork = input.fork()
            && let Ok(binary) = fork.parse()
        {
            input.advance_to(&fork);
            Ok(ExprSuffix::Binary(binary))
        } else if let fork = input.fork()
            && let Ok(call) = fork.parse()
        {
            input.advance_to(&fork);
            Ok(ExprSuffix::Call(call))
        } else if let fork = input.fork()
            && let Ok(cast) = fork.parse()
        {
            input.advance_to(&fork);
            Ok(ExprSuffix::Cast(cast))
        } else if let fork = input.fork()
            && let Ok(dot) = fork.parse()
        {
            input.advance_to(&fork);
            Ok(ExprSuffix::Dot(dot))
        } else if let Ok(index) = input.parse() {
            Ok(ExprSuffix::Index(index))
        } else if input.parse::<Token![?]>().is_ok() {
            Ok(ExprSuffix::Try)
        } else {
            Err(input.error("TODO: error strings"))
        }
    }
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
        let fork = input.fork();
        let kind = fork.parse()?;
        let expr = fork.parse()?;
        input.advance_to(&fork);
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

impl Parse for ExprSuffixCall {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        let content;
        parenthesized!(content in fork);
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?
            .into_iter()
            .collect();
        input.advance_to(&fork);
        Ok(ExprSuffixCall { args })
    }
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
        let fork = input.fork();
        _ = fork.parse::<Token![as]>()?;
        let ty_parsed = fork.parse::<syn::Type>()?;
        input.advance_to(&fork);

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
        let fork = input.fork();
        _ = fork.parse::<Token![.]>()?;
        if fork.parse::<Token![await]>().is_ok() {
            input.advance_to(&fork);
            Ok(ExprSuffixDot::Await)
        } else if let Ok(method_call) = fork.parse() {
            input.advance_to(&fork);
            Ok(ExprSuffixDot::MethodCall(method_call))
        } else if let Ok(field) = fork.parse() {
            input.advance_to(&fork);
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

            let args = Punctuated::<Expr, Token![,]>::parse_terminated
                .parse2(inner_cursor.token_stream())?
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

impl Parse for ExprSuffixIndex {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        let content;
        bracketed!(content in fork);
        let index = content.parse()?;
        input.advance_to(&fork);
        Ok(ExprSuffixIndex { index })
    }
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
    use std::fmt::Debug;

    use proc_macro2::{TokenStream, TokenTree};
    use quote::{ToTokens, format_ident, quote};
    use syn::{
        Result,
        parse::{Parse, ParseBuffer, Parser},
        parse_str,
    };

    use crate::{
        BlockKind, Closure, Expr, ExprBase, ExprPrefix, ExprSuffix, ExprSuffixBinary,
        ExprSuffixBinaryKind, ExprSuffixCall, ExprSuffixCast, ExprSuffixDot, ExprSuffixDotField,
        ExprSuffixDotMethodCall, ExprSuffixIndex,
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

    /// Asserts that the tokens in [`TokenStream`], which must be a partially valid value of `T` as tokens, are not
    /// consumed when parsing an invalid value of `T`.
    #[track_caller]
    fn assert_respect<T: Parse + Debug>(tokens: TokenStream) {
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

    fn expr(base: ExprBase) -> Expr {
        Expr {
            attrs: TokenStream::new(),
            prefixes: vec![],
            base,
            suffixes: vec![],
        }
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
        assert_token_stream(closure.pats, quote! { ident: Type });

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
    fn expr_parse() {
        let expr = parse_str::<Expr>("0123").unwrap();
        assert!(expr.attrs.is_empty());
        assert!(expr.prefixes.is_empty());
        assert!(matches!(expr.base, ExprBase::Lit(_)));
        assert!(expr.suffixes.is_empty());

        let expr = parse_str::<Expr>("#[meta] 0123").unwrap();
        assert_token_stream(expr.attrs, quote! { #[meta] });
        assert!(expr.prefixes.is_empty());
        assert!(matches!(expr.base, ExprBase::Lit(_)));
        assert!(expr.suffixes.is_empty());

        let expr = parse_str::<Expr>("#[meta(foo = [0123])] 0123").unwrap();
        assert_token_stream(expr.attrs, quote! { #[meta(foo = [0123])] });

        let expr = parse_str::<Expr>("#[meta] #[meta_foo] 0123").unwrap();
        assert_token_stream(expr.attrs, quote! { #[meta] #[meta_foo] });

        let expr = parse_str::<Expr>("#![meta] 0123").unwrap();
        assert_token_stream(expr.attrs, quote! { #![meta] });

        let expr = parse_str::<Expr>("!0123").unwrap();
        assert_eq!(expr.prefixes.len(), 1);
        assert!(matches!(expr.prefixes[0], ExprPrefix::Not));

        let expr = parse_str::<Expr>("!!!0123").unwrap();
        assert_eq!(expr.prefixes.len(), 3);
        assert!(matches!(expr.prefixes[0], ExprPrefix::Not));
        assert!(matches!(expr.prefixes[1], ExprPrefix::Not));
        assert!(matches!(expr.prefixes[2], ExprPrefix::Not));

        let expr = parse_str::<Expr>("0123?").unwrap();
        assert_eq!(expr.suffixes.len(), 1);
        assert!(matches!(expr.suffixes[0], ExprSuffix::Try));

        let expr = parse_str::<Expr>("0123???").unwrap();
        assert_eq!(expr.suffixes.len(), 3);
        assert!(matches!(expr.suffixes[0], ExprSuffix::Try));
        assert!(matches!(expr.suffixes[1], ExprSuffix::Try));
        assert!(matches!(expr.suffixes[2], ExprSuffix::Try));

        let expr = parse_str::<Expr>("#[meta] #[meta] !!!0123???").unwrap();
        assert_token_stream(expr.attrs, quote! { #[meta] #[meta] });
        assert_eq!(expr.prefixes.len(), 3);
        assert!(matches!(expr.prefixes[0], ExprPrefix::Not));
        assert!(matches!(expr.prefixes[1], ExprPrefix::Not));
        assert!(matches!(expr.prefixes[2], ExprPrefix::Not));
        assert!(matches!(expr.base, ExprBase::Lit(_)));
        assert_eq!(expr.suffixes.len(), 3);
        assert!(matches!(expr.suffixes[0], ExprSuffix::Try));
        assert!(matches!(expr.suffixes[1], ExprSuffix::Try));
        assert!(matches!(expr.suffixes[2], ExprSuffix::Try));
    }

    #[test]
    fn expr_print() {
        assert_to_tokens(
            Expr {
                attrs: quote! { #![meta] #[meta] },
                prefixes: vec![ExprPrefix::Not, ExprPrefix::Neg],
                base: ExprBase::Lit(quote! { 0123 }),
                suffixes: vec![
                    ExprSuffix::Try,
                    ExprSuffix::Binary(ExprSuffixBinary {
                        kind: ExprSuffixBinaryKind::Add,
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
    fn expr_prefix_parse() {
        assert!(matches!(parse_str("*").unwrap(), ExprPrefix::Deref));
        assert!(matches!(parse_str("-").unwrap(), ExprPrefix::Neg));
        assert!(matches!(parse_str("!").unwrap(), ExprPrefix::Not));
        assert!(matches!(
            parse_str("&raw const").unwrap(),
            ExprPrefix::RawConst,
        ));
        assert!(matches!(parse_str("&raw mut").unwrap(), ExprPrefix::RawMut));
        assert!(matches!(parse_str("&").unwrap(), ExprPrefix::Ref));
        assert!(matches!(parse_str("&mut").unwrap(), ExprPrefix::RefMut));

        _ = parse_str::<ExprPrefix>("&raw").unwrap_err();
    }

    #[test]
    fn expr_prefix_print() {
        assert_to_tokens(ExprPrefix::Deref, "*");
        assert_to_tokens(ExprPrefix::Neg, "-");
        assert_to_tokens(ExprPrefix::Not, "!");
        assert_to_tokens(ExprPrefix::RawConst, "&raw const");
        assert_to_tokens(ExprPrefix::RawMut, "&raw mut");
        assert_to_tokens(ExprPrefix::Ref, "&");
        assert_to_tokens(ExprPrefix::RefMut, "&mut");
    }

    #[test]
    fn expr_prefix_respect() {
        assert_respect::<ExprPrefix>(quote! { &raw });
    }

    #[test]
    fn expr_suffix_parse() {
        assert!(matches!(parse_str("+ 0").unwrap(), ExprSuffix::Binary(_)));
        assert!(matches!(parse_str("(0123)").unwrap(), ExprSuffix::Call(_)));
        assert!(matches!(parse_str("as Type").unwrap(), ExprSuffix::Cast(_)));
        assert!(matches!(parse_str(".ident").unwrap(), ExprSuffix::Dot(_)));
        assert!(matches!(parse_str("[0123]").unwrap(), ExprSuffix::Index(_)));
        assert!(matches!(parse_str("?").unwrap(), ExprSuffix::Try));
    }

    #[test]
    fn expr_suffix_print() {
        assert_to_tokens(ExprSuffix::Try, "?");
    }

    #[test]
    fn expr_suffix_respect() {
        assert_respect::<ExprSuffix>(quote! { + });
        assert_respect::<ExprSuffix>(quote! { as });
        assert_respect::<ExprSuffix>(quote! { . });
    }

    #[test]
    fn expr_suffix_binary_parse() {
        let binary = parse_str::<ExprSuffixBinary>("+ 0123").unwrap();
        assert!(matches!(binary.kind, ExprSuffixBinaryKind::Add));
        assert!(matches!(binary.expr.base, ExprBase::Lit(_)));

        let binary = parse_str::<ExprSuffixBinary>("<<= \"foo\"").unwrap();
        assert!(matches!(binary.kind, ExprSuffixBinaryKind::ShlAssign));
        assert!(matches!(binary.expr.base, ExprBase::Lit(_)));

        _ = parse_str::<ExprSuffixBinary>("<Type>").unwrap_err();
    }

    #[test]
    fn expr_suffix_binary_print() {
        assert_to_tokens(
            ExprSuffixBinary {
                kind: ExprSuffixBinaryKind::Add,
                expr: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
            },
            "+ 0123",
        );
        assert_to_tokens(
            ExprSuffixBinary {
                kind: ExprSuffixBinaryKind::ShlAssign,
                expr: Box::new(expr(ExprBase::Lit(quote! { "foo" }))),
            },
            "<<= \"foo\"",
        );
    }

    #[test]
    fn expr_suffix_binary_respect() {
        assert_respect::<ExprSuffixBinary>(quote! { + });
        assert_respect::<ExprSuffixBinary>(quote! { <<= });
    }

    #[test]
    fn expr_suffix_binary_kind_parse() {
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
    fn expr_suffix_binary_kind_print() {
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
    fn expr_suffix_binary_kind_respect() {
        assert_respect::<ExprSuffixBinaryKind>(quote! { ! });
    }

    #[test]
    fn expr_suffix_call_parse() {
        assert!(parse_str::<ExprSuffixCall>("()").unwrap().args.is_empty());

        let call = parse_str::<ExprSuffixCall>("(0123)").unwrap();
        assert_eq!(call.args.len(), 1);
        assert!(matches!(call.args[0].base, ExprBase::Lit(_)));

        let call = parse_str::<ExprSuffixCall>("(\"foo\", \"bar\",)").unwrap();
        assert_eq!(call.args.len(), 2);
        assert!(matches!(call.args[0].base, ExprBase::Lit(_)));
        assert!(matches!(call.args[1].base, ExprBase::Lit(_)));
    }

    #[test]
    fn expr_suffix_call_print() {
        assert_to_tokens(ExprSuffixCall { args: vec![] }, "()");
        assert_to_tokens(
            ExprSuffixCall {
                args: vec![expr(ExprBase::Lit(quote! { 0123 }))],
            },
            "(0123)",
        );
        assert_to_tokens(
            ExprSuffixCall {
                args: vec![
                    expr(ExprBase::Lit(quote! { "foo" })),
                    expr(ExprBase::Lit(quote! { "bar" })),
                ],
            },
            "(\"foo\", \"bar\")",
        );
    }

    #[test]
    fn expr_suffix_cast_parse() {
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

        _ = parse_str::<ExprSuffixCast>("Type").unwrap_err();
        _ = parse_str::<ExprSuffixCast>("<Type>").unwrap_err();
        _ = parse_str::<ExprSuffixCast>("as <Type as Trait>").unwrap_err();
    }

    #[test]
    fn expr_suffix_cast_print() {
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
    fn expr_suffix_cast_respect() {
        assert_respect::<ExprSuffixCast>(quote! { as });
        assert_respect::<ExprSuffixCast>(quote! { as :: });
        assert_respect::<ExprSuffixCast>(quote! { as module:: });
    }

    #[test]
    fn expr_suffix_dot_parse() {
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
    fn expr_suffix_dot_print() {
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
    fn expr_suffix_dot_respect() {
        assert_respect::<ExprSuffixDot>(quote! { . });
    }

    #[test]
    fn expr_suffix_dot_field_parse() {
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
    fn expr_suffix_dot_field_print() {
        assert_to_tokens(ExprSuffixDotField::Named(format_ident!("ident")), "ident");
        assert_to_tokens(ExprSuffixDotField::Unnamed(0123), "123");
    }

    #[test]
    fn expr_suffix_dot_method_call_parse() {
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
    fn expr_suffix_dot_method_call_print() {
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
    fn expr_suffix_dot_method_call_respect() {
        assert_respect::<ExprSuffixDotMethodCall>(quote! { method });
    }

    #[test]
    fn expr_suffix_index_parse() {
        assert!(matches!(
            parse_str::<ExprSuffixIndex>("[0123]").unwrap().index.base,
            ExprBase::Lit(_),
        ));
        assert!(matches!(
            parse_str::<ExprSuffixIndex>("[\"0123\"]")
                .unwrap()
                .index
                .base,
            ExprBase::Lit(_),
        ));

        _ = parse_str::<ExprSuffixIndex>("").unwrap_err();
        _ = parse_str::<ExprSuffixIndex>("[0123 +]").unwrap_err();
    }

    #[test]
    fn expr_suffix_index_print() {
        assert_to_tokens(
            ExprSuffixIndex {
                index: Box::new(expr(ExprBase::Lit(quote! { 0123 }))),
            },
            "[0123]",
        );
        assert_to_tokens(
            ExprSuffixIndex {
                index: Box::new(expr(ExprBase::Lit(quote! { "foo" }))),
            },
            "[\"foo\"]",
        );
    }
}
