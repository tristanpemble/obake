use std::convert::{TryFrom, TryInto};
use std::ops::Range;

use syn::{braced, Expr, Lit, parenthesized, RangeLimits, Token};
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;

use crate::internal::*;

const OBAKE: &str = "obake";

impl Parse for VersionAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let version_int = input.parse::<syn::LitInt>()?;
        let span = version_int.span();
        let version = version_int.base10_parse()
                                 .map_err(|err| syn::Error::new(version_int.span(), err))?;

        Ok(Self { version, span })
    }
}

impl Parse for CfgAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr = input.parse::<Expr>()?;

        match expr {
            Expr::Range(req_range) => {
                let span = req_range.span();
                let start = match req_range.from {
                    Some(v) => match *v {
                        Expr::Lit(expr_lit) => match expr_lit.lit {
                            Lit::Int(lit_int) => {
                                lit_int.base10_parse()
                                       .map_err(|err| syn::Error::new(lit_int.span(), err))?
                            }
                            _ => return Err(syn::Error::new(span, "expected integer literal for range start"))
                        },
                        _ => return Err(syn::Error::new(span, "expected integer literal for range start"))
                    },
                    _ => u32::MIN,
                };
                let end = match req_range.to {
                    Some(v) => match *v {
                        Expr::Lit(expr_lit) => match expr_lit.lit {
                            Lit::Int(lit_int) => {
                                lit_int.base10_parse()
                                       .map_err(|err| syn::Error::new(lit_int.span(), err))?
                            }
                            _ => return Err(syn::Error::new(span, "expected integer literal for range end")),
                        },
                        _ => return Err(syn::Error::new(span, "expected integer literal for range end")),
                    },
                    _ => u32::MAX,
                };
                let req = Range {
                    start,
                    end: match req_range.limits {
                        RangeLimits::HalfOpen(_) => end,
                        RangeLimits::Closed(_) => end + 1,
                    },
                };
                Ok(Self { req, span })
            }
            Expr::Lit(expr_lit) => match expr_lit.lit {
                Lit::Int(int) => {
                    let start = int.base10_parse()
                                   .map_err(|err| syn::Error::new(int.span(), err))?;
                    let req = Range {
                        start,
                        end: start + 1,
                    };
                    let span = int.span();
                    Ok(Self { req, span })
                }
                _ => return Err(syn::Error::new(expr_lit.span(), "expected a range or an int literal"))
            },
            _ => return Err(syn::Error::new(expr.span(), "expected a range or an int literal"))
        }
    }
}

impl Parse for ObakeAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<syn::Ident>()?;

        Ok(match ident {
            _ if ident == "version" => {
                let content;
                parenthesized!(content in input);
                Self::Version(content.parse()?)
            }
            _ if ident == "cfg" => {
                let content;
                parenthesized!(content in input);
                Self::Cfg(content.parse()?)
            }
            _ if ident == "inherit" => Self::Inherit(InheritAttr { span: ident.span() }),
            _ if ident == "derive" => {
                let content;
                parenthesized!(content in input);
                Self::Derive(DeriveAttr {
                    span: ident.span(),
                    tokens: content.parse()?,
                })
            }
            #[cfg(feature = "serde")]
            _ if ident == "serde" => {
                let content;
                parenthesized!(content in input);
                Self::Serde(SerdeAttr {
                    span: ident.span(),
                    tokens: content.parse()?,
                })
            }
            _ => {
                return Err(syn::Error::new(
                    ident.span(),
                    "unrecognised `obake` helper attribute",
                ));
            }
        })
    }
}

impl TryFrom<syn::Attribute> for ObakeAttribute {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self> {
        attr.parse_args()
    }
}

impl TryFrom<syn::Attribute> for VersionedAttribute {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self> {
        attr.path.get_ident().map_or_else(
            || Ok(Self::Attribute(attr.clone())),
            |ident| {
                if ident == OBAKE {
                    Ok(Self::Obake(attr.clone().try_into()?))
                } else {
                    Ok(Self::Attribute(attr.clone()))
                }
            },
        )
    }
}

impl Parse for VersionedAttributes {
    fn parse(input: ParseStream) -> Result<VersionedAttributes> {
        let attrs = input
            .call(syn::Attribute::parse_outer)?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { attrs })
    }
}

impl Parse for VersionedField {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.parse()?,
            vis: input.parse()?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

impl Parse for VersionedFields {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let brace_token = braced!(content in input);

        Ok(Self {
            brace_token,
            fields: content.parse_terminated(VersionedField::parse)?,
        })
    }
}

impl Parse for VersionedVariantFields {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self::Unit);
        }

        let lookahead = input.lookahead1();
        Ok(if lookahead.peek(syn::token::Paren) {
            Self::Unnamed(input.parse()?)
        } else if lookahead.peek(syn::token::Brace) {
            Self::Named(input.parse()?)
        } else {
            Self::Unit
        })
    }
}

impl Parse for VersionedVariant {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.parse()?,
            ident: input.parse()?,
            fields: input.parse()?,
        })
    }
}

impl Parse for VersionedVariants {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let brace_token = braced!(content in input);

        Ok(Self {
            brace_token,
            variants: content.parse_terminated(VersionedVariant::parse)?,
        })
    }
}

impl Parse for VersionedStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            struct_token: input.parse()?,
            ident: input.parse()?,
            fields: input.parse()?,
        })
    }
}

impl Parse for VersionedEnum {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            enum_token: input.parse()?,
            ident: input.parse()?,
            variants: input.parse()?,
        })
    }
}

impl Parse for VersionedItemKind {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            Ok(Self::Struct(input.parse()?))
        } else if lookahead.peek(Token![enum]) {
            Ok(Self::Enum(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for VersionedItem {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.parse()?,
            vis: input.parse()?,
            kind: input.parse()?,
        })
    }
}
