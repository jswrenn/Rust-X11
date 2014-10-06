#![crate_name = "syntax_extensions"]
#![crate_type = "dylib"]
#![feature(plugin_registrar, phase)]

extern crate syntax;
extern crate rustc;

use syntax::ast;
use syntax::codemap::{Span, Spanned};
use syntax::fold::Folder;
use syntax::parse;
use syntax::parse::token;
use syntax::parse::token::get_name;
use syntax::print::pprust;
use syntax::ext::base::{DummyResult, ExtCtxt, Modifier, MacExpr, MacResult};
use syntax::ptr::P as Ptr;

#[plugin_registrar]
pub fn registrar(reg: &mut rustc::plugin::Registry) {
  reg.register_syntax_extension(token::intern("change_ident_to"),
                                Modifier(box expand_change_ident_to));
  reg.register_syntax_extension(token::intern("inner_attributes"),
                                Modifier(box expand_inner_attributes));
  reg.register_macro("CamelCase", expand_camel_case);
  reg.register_macro("snake_case", expand_snake_case);
}

fn expand_inner_attributes(context: &mut ExtCtxt, span: Span, metaitem: &ast::MetaItem,
                           item: Ptr<ast::Item>) -> Ptr<ast::Item> {
    match metaitem.node {
        ast::MetaWord(..) => {
            match item.node {
                ast::ItemImpl(ref generics, ref maybe_trait, ref ty_ptr, ref impl_items) => {
                    let mut new_impl_items = Vec::with_capacity(impl_items.len());
                    for impl_item in impl_items.iter() {
                        match *impl_item {
                            ast::MethodImplItem(ref ptr) => {
                                let ref old_method = *ptr;
                                match old_method.node {
                                    ast::MethDecl(old_ident, ref generics, abi, ref explicit_self,
                                                  fn_style, ref decl, ref block, vis) => {
                                        let (new_name, new_attrs) = new_name_and_attrs(context, span, old_ident.name, &old_method.attrs);
                                        let new_method = Ptr(ast::Method {
                                            attrs: new_attrs,
                                            id: old_method.id.clone(),
                                            span: old_method.span.clone(),
                                            node: ast::MethDecl(ast::Ident::new(new_name),
                                                                  generics.clone(), abi, explicit_self.clone(),
                                                                  fn_style, decl.clone(), block.clone(), vis)
                                        });
                                        let new_impl_item = ast::MethodImplItem(new_method);
                                        new_impl_items.push(new_impl_item);
                                    }
                                    //FIXME: This case should be handled eventually.
                                    ast::MethMac(..) => fail!("Handling of macros in method position not yet implemented by “inner_attributes”.")
                                }
                            }
                            ref some_impl_item @ _ => new_impl_items.push(some_impl_item.clone())
                        }
                    }
                    Ptr(ast::Item {
                        node:  ast::ItemImpl(generics.clone(), maybe_trait.clone(), ty_ptr.clone(), new_impl_items),
                        .. (*item).clone()
                    })
                }
                ast::ItemEnum(ref enum_def, ref generics) => {
                    let mut new_enum_def = ast::EnumDef { variants: Vec::with_capacity(enum_def.variants.len()) };
                    for old_variant_ptr in enum_def.variants.iter() {
                        let ref old_variant = old_variant_ptr.node;
                        let (new_name, new_attrs) = new_name_and_attrs(context, span, old_variant.name.name, &old_variant.attrs);
                        let new_variant_ptr = Ptr(Spanned {
                            node: ast::Variant_ {
                                name: ast::Ident::new(new_name),
                                attrs: new_attrs,
                                .. old_variant.clone()
                            },
                            span: old_variant_ptr.span
                        });
                        new_enum_def.variants.push(new_variant_ptr);
                    }
                    Ptr(ast::Item {
                        ident: item.ident,
                        attrs: item.attrs.clone(),
                        id:    item.id,
                        node:  ast::ItemEnum(new_enum_def, generics.clone()),
                        vis:   item.vis,
                        span:  item.span
                    })
                }
                _ => {
                    context.span_err(span, "“inner_attributes” does not handle this type of item.");
                    item.clone()
                }
            }
        }
        _ => {
            context.span_err(span, "“inner_attributes” is used without arguments.");
            item
        }
    }
}

fn new_name_and_attrs(context: &mut ExtCtxt, span: Span, old_name: ast::Name, old_attrs: &Vec<ast::Attribute>) -> (ast::Name, Vec<ast::Attribute>) {
    let change_ident_str = get_name(token::intern("change_ident_to"));
    let mut new_name = old_name;
    let mut new_attrs = Vec::with_capacity(old_attrs.len());
    for attr in old_attrs.iter() {
        match attr.node.value.node {
            ast::MetaList(ref interned_str, _) => {
                if change_ident_str == *interned_str {
                    new_name = token::intern(ident_from_meta_item(context, span, &*attr.node.value).as_slice());
                }
                else {
                    new_attrs.push(attr.clone())
                }
            }
            _ => new_attrs.push(attr.clone())
        }
    }
    new_attrs.shrink_to_fit();
    (new_name, new_attrs)
}

fn ident_from_meta_item(context: &mut ExtCtxt, span: Span, metaitem: &ast::MetaItem) -> String {
    let mut new_name = String::new();
    match metaitem.node {
        ast::MetaList(_, ref vec) => {
            if vec.is_empty() {
                context.span_err(span, "“change_ident_to” expects a non-empty argument list.");
                new_name
            }
            else {
                for meta_itm in vec.iter() {
                    match meta_itm.node {
                        ast::MetaWord(ref interned_str) => new_name.push_str(interned_str.get()),
                        ast::MetaList(ref interned_str, ref vec) => {
                            let convert: fn(&[Ascii]) -> String;
                            let str = interned_str.get();
                            if str == "CamelCase" {
                                convert = snake_to_camel;
                            }
                            else if str == "snake_case" {
                                convert = camel_to_snake;
                            }
                            else {
                                let error_msg = format!("Invoking unknown identifier function {} in “change_ident_to”.", str);
                                context.span_err(span, error_msg.as_slice());
                                return new_name
                            }
                            let error_msg = format!("Invocation of {} in “change_ident_to” may only be done with exactly one identifier.", str);
                            if vec.len() == 1 {
                                let ref meta_itm = vec[0];
                                match meta_itm.node {
                                    ast::MetaWord(ref interned_string) => {
                                        let str = interned_string.get();
                                        let converted_string = unsafe { convert(str.to_ascii_nocheck()) };
                                        new_name.push_str(converted_string.as_slice())
                                    }
                                    _ => {
                                        context.span_err(span, error_msg.as_slice());
                                        return new_name
                                    }
                                }
                            }
                            else {
                                context.span_err(span, error_msg.as_slice());
                                return new_name
                            }
                        }
                        _ => {
                            context.span_err(span,
                                             "“change_ident_to” expects either an identifier, sequence of identifiers, an identifier applied to CamelCase/snake_case (i.e. “snake_case(identifier)”), or a sequence of applications of CamelCase/snake_case.");
                            return new_name
                        }
                    }
                }
                new_name
            }
        }
        _ => {
            context.span_err(span, "“change_ident_to” expects arguments.");
            new_name
        }
    }

}

fn expand_change_ident_to(context: &mut ExtCtxt, span: Span, metaitem: &ast::MetaItem,
                          old_item: Ptr<ast::Item>) -> Ptr<ast::Item> {
    let new_name = ident_from_meta_item(context, span, metaitem);
    let new_item = Ptr(ast::Item {
        ident: ast::Ident::new(token::intern(new_name.as_slice())),
        attrs: old_item.attrs.clone(),
        id: old_item.id.clone(),
        node: old_item.node.clone(),
        vis: old_item.vis.clone(),
        span: old_item.span.clone()
    });
    new_item
}

fn expand_path_transform(context: &mut ExtCtxt, span: Span, tokens: &[ast::TokenTree], convert: fn(&[Ascii]) -> String) -> Box<MacResult + 'static> {
    let mut parser = parse::new_parser_from_tts(context.parse_sess(), context.cfg(), tokens.to_vec());
    let expr = context.expander().fold_expr(parser.parse_expr());
    if !parser.eat(&token::EOF) {
        context.span_err(parser.span, "Expected a single expression.");
        return DummyResult::expr(span)
    }
    match expr.node {
        ast::ExprPath(ref old_path) => {
            let mut new_segments = Vec::with_capacity(old_path.segments.len());
            for old_segment in old_path.segments.iter() {
                let new_str = convert(unsafe {old_segment.identifier.as_str().to_ascii_nocheck()});
                let new_ident = ast::Ident::new(token::intern(new_str.as_slice()));
                new_segments.push(
                    ast::PathSegment {
                        identifier: new_ident,
                        lifetimes: old_segment.lifetimes.clone(),
                        types: old_segment.types.clone()
                    })
            }
            let new_path = ast::Path {
                span: old_path.span,
                global: old_path.global,
                segments: new_segments
            };
            MacExpr::new(Ptr(
                    ast::Expr {
                        id: expr.id,
                        node: ast::ExprPath(new_path),
                        span: expr.span
                }))
        }
        _ => {
            let err_msg = format!("Unsupported expression given to camel_case!: “{}”", pprust::expr_to_string(&*expr));
            context.span_err(parser.span, err_msg.as_slice());
            DummyResult::expr(span)
        }
    }
}

#[inline]
fn expand_camel_case(context: &mut ExtCtxt, span: Span, tokens: &[ast::TokenTree]) -> Box<MacResult + 'static> {
    expand_path_transform(context, span, tokens, snake_to_camel)
}

#[inline]
fn expand_snake_case(context: &mut ExtCtxt, span: Span, tokens: &[ast::TokenTree]) -> Box<MacResult + 'static> {
    expand_path_transform(context, span, tokens, camel_to_snake)
}

//These two conversion functions are intentionally simple and make assumptions
//suited for their specific use cases in this file.
fn snake_to_camel(xs: &[Ascii]) -> String {
    let mut i = xs.iter();
    let mut result = String::with_capacity(i.len());
    match i.next() {
        None => (),
        Some(c) => {
            result.push(c.to_uppercase().to_char());
            let mut last_was_underscore = false;
            for x in i {
                if *x == unsafe { '_'.to_ascii_nocheck() } {
                    last_was_underscore = true
                }
                else {
                    if last_was_underscore {
                        last_was_underscore = false;
                        result.push(x.to_uppercase().to_char())
                    }
                    else {
                        result.push(x.to_char())
                    }
                }
            }
        }
    }
    result
}

fn camel_to_snake(xs: &[Ascii]) -> String {
    let mut i = xs.iter();
    let mut result = String::with_capacity(i.len());
    match i.next() {
        None => (),
        Some(c) => {
            result.push(c.to_lowercase().to_char());
            for x in i {
                if x.is_uppercase() {
                    result.push('_');
                    result.push(x.to_lowercase().to_char())
                }
                else {
                    result.push(x.to_char())
                }
            }
        }
    }
    result
}

