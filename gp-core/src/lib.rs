//! # PARI/GP
//! 
//! This crate only contains the `gp!{}` macro. It is not meant to be added into your project.
//! Refer to the `pari-gp` crate instead.

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Ident, Span, TokenStream as TokenStream2, TokenTree};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{quote, quote_spanned};

enum Segment {
    Literal(String),
    Var(Ident)
}

#[proc_macro]
pub fn gp(input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();

    let gp_crate = match crate_name("pari-gp") {
        Ok(FoundCrate::Itself) => quote!(::pari_gp),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        },
        Err(_) => {
            return quote! {
                compile_error!("`gp!` requires the `pari-gp` crate; add `pari-gp` as a dependency instead of `gp-core`");
            }.into()
        }
    };

    let mut segments = Vec::new();
    if let Err(error_tokens) = collect_segments(input2, &mut segments) {
        return error_tokens.into();
    }

    let token_segments = segments.into_iter().map(|s|
        match s {
            Segment::Literal(s) => quote! {
                __gp_source.push_str(#s);
            },
            Segment::Var(ident) => quote_spanned! { ident.span() =>
                __gp_source.push(' ');
                __gp_source.push_str(&#gp_crate::__runtime::IntoGp::into(&#ident));
                __gp_source.push(' ');
            },
    });

    let result = quote!{
        {
            let mut __gp_source = ::std::string::String::new();
            #(#token_segments)*
            match #gp_crate::__runtime::run(&__gp_source) {
                ::std::result::Result::Ok(__gp_output) => {
                    match #gp_crate::__runtime::try_from_gp(&__gp_output) {
                        ::std::result::Result::Ok(__gp_type) => __gp_type,
                        ::std::result::Result::Err(__parse_error) => {
                            let location = format!("{}:{}:{}", file!(), line!(), column!());
                            eprintln!("{}", __parse_error.pretty(&location));
                            ::std::process::exit(1);
                        }
                    }
                }
                ::std::result::Result::Err(__gp_error) => {
                    let location = format!("{}:{}:{}", file!(), line!(), column!());
                    eprintln!("{}", __gp_error.pretty(&location));
                    ::std::process::exit(1);
                }
            }
        }
    };

    result.into()
}

fn collect_segments(ts: TokenStream2, segments: &mut Vec<Segment>) -> Result<(), TokenStream2> {
    let mut pending = TokenStream2::new();
    let mut iter = ts.into_iter();
 
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Punct(p) if p.as_char() == '@' => {
                flush(&mut pending, segments);

                match iter.next() {
                    Some(TokenTree::Ident(ident)) => segments.push(Segment::Var(ident)),
                    Some(other) => {
                        return Err(quote_spanned! { other.span() =>
                            compile_error!("expected an identifier after `@`");
                        });
                    }
                    None => {
                        return Err(quote_spanned! { p.span() =>
                            compile_error!("expected an identifier after `@`");
                        });
                    }
                }
            }

            // we need to recursively do this for inner groups
            TokenTree::Group(g) => {
                flush(&mut pending, segments);

                let (open, close) = delimiters(g.delimiter());

                if !open.is_empty() {
                    segments.push(Segment::Literal(open));
                }

                collect_segments(g.stream(), segments)?;

                if !close.is_empty() {
                    segments.push(Segment::Literal(close));
                }
            }

            other => pending.extend(std::iter::once(other)),
        }
    }

    flush(&mut pending, segments);

    Ok(())
}

// if the parser found an @identifier or a start of a group, it needs to dump all the previous
// tokens it had collected and just map it as a plain string as is
fn flush(pending: &mut TokenStream2, segments: &mut Vec<Segment>) {
    if !pending.is_empty() {
        segments.push(Segment::Literal(std::mem::take(pending).to_string()));
    }
}
 
fn delimiters(d: Delimiter) -> (String, String) {
    let (s1, s2) = match d {
        Delimiter::Parenthesis => ("(", ")"),
        Delimiter::Brace       => ("{", "}"),
        Delimiter::Bracket     => ("[", "]"),
        Delimiter::None        => ("", ""),
    };

    (s1.to_string(), s2.to_string())
}
