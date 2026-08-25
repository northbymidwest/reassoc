//! The handful of syntax shapes the rewriter emits, built directly rather
//! than `quote!`d and re-parsed (see `rewrite.rs` for why). Every token is
//! placed at the span the caller gives, which is the operator's, and that is
//! what anchors the diagnostics on it. Coupling to syn's field layout lives
//! here and nowhere else.

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, Token};

/// `::a::b::c`, with the leading colon and separators at `span`; each ident keeps
/// its own span.
pub fn path(span: Span, segments: impl IntoIterator<Item = syn::Ident>) -> Expr {
    let mut path = syn::Path {
        leading_colon: Some(Token![::](span)),
        segments: Punctuated::new(),
    };
    for ident in segments {
        if !path.segments.is_empty() {
            path.segments.push_punct(Token![::](span));
        }
        path.segments.push_value(syn::PathSegment::from(ident));
    }
    Expr::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path,
    })
}

/// A bare identifier as an expression.
pub fn ident(ident: syn::Ident) -> Expr {
    Expr::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: syn::Path::from(ident),
    })
}

/// `#[..] func(args..)`, parens at `span`.
pub fn call(
    span: Span,
    func: Expr,
    args: impl IntoIterator<Item = Expr>,
    attrs: Vec<Attribute>,
) -> Expr {
    Expr::Call(syn::ExprCall {
        attrs,
        func: Box::new(func),
        paren_token: syn::token::Paren(span),
        args: args.into_iter().collect(),
    })
}

/// `&mut place`.
pub fn ref_mut(span: Span, place: Expr) -> Expr {
    Expr::Reference(syn::ExprReference {
        attrs: Vec::new(),
        and_token: Token![&](span),
        mutability: Some(Token![mut](span)),
        expr: Box::new(place),
    })
}

/// `(elem,)`.
pub fn tuple1(span: Span, elem: Expr) -> Expr {
    Expr::Tuple(syn::ExprTuple {
        attrs: Vec::new(),
        paren_token: syn::token::Paren(span),
        elems: one(span, elem),
    })
}

/// `(pat,)`.
pub fn pat_tuple1(span: Span, pat: syn::Pat) -> syn::Pat {
    syn::Pat::Tuple(syn::PatTuple {
        attrs: Vec::new(),
        paren_token: syn::token::Paren(span),
        elems: one(span, pat),
    })
}

/// A plain binding pattern.
pub fn bind(ident: syn::Ident) -> syn::Pat {
    syn::Pat::Ident(syn::PatIdent {
        attrs: Vec::new(),
        by_ref: None,
        mutability: None,
        ident,
        subpat: None,
    })
}

/// `{ stmt; }`.
pub fn block1(span: Span, stmt: Expr) -> Expr {
    Expr::Block(syn::ExprBlock {
        attrs: Vec::new(),
        label: None,
        block: syn::Block {
            brace_token: syn::token::Brace(span),
            stmts: vec![syn::Stmt::Expr(stmt, Some(Token![;](span)))],
        },
    })
}

/// `match scrutinee { pat => body }`.
pub fn match1(span: Span, scrutinee: Expr, pat: syn::Pat, body: Expr) -> Expr {
    Expr::Match(syn::ExprMatch {
        attrs: Vec::new(),
        match_token: Token![match](span),
        expr: Box::new(scrutinee),
        brace_token: syn::token::Brace(span),
        arms: vec![syn::Arm {
            attrs: Vec::new(),
            pat,
            fat_arrow_token: Token![=>](span),
            body: Box::new(body),
            comma: None,
        }],
    })
}

/// `#[allow(lint)]`.
pub fn allow(span: Span, lint: &str) -> Attribute {
    let lint = syn::Ident::new(lint, span);
    Attribute {
        pound_token: Token![#](span),
        style: syn::AttrStyle::Outer,
        bracket_token: syn::token::Bracket(span),
        meta: syn::Meta::List(syn::MetaList {
            path: syn::Path::from(syn::Ident::new("allow", span)),
            delimiter: syn::MacroDelimiter::Paren(syn::token::Paren(span)),
            tokens: quote::quote_spanned!(span=> #lint),
        }),
    }
}

/// One element with its trailing comma.
fn one<T>(span: Span, elem: T) -> Punctuated<T, Token![,]> {
    let mut elems = Punctuated::new();
    elems.push_value(elem);
    elems.push_punct(Token![,](span));
    elems
}
