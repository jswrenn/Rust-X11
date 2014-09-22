#![crate_name = "syntax_extensions"]
#![crate_type = "dylib"]
#![feature(plugin_registrar, phase)]

extern crate syntax;
extern crate rustc;

use syntax::ast;
use syntax::codemap::Span;
use syntax::parse::token;
use syntax::parse::token::get_name;
use syntax::ext::base::{ItemModifier, ExtCtxt};
use syntax::ptr::P as Ptr;

#[plugin_registrar]
pub fn registrar(reg: &mut rustc::plugin::Registry) {
  reg.register_syntax_extension(token::intern("change_ident_to"),
                                ItemModifier(box expand_change_ident_to));
  reg.register_syntax_extension(token::intern("method_modifiers"),
                                ItemModifier(box expand_method_modifiers));
}

fn expand_method_modifiers(context: &mut ExtCtxt, span: Span, metaitem: &ast::MetaItem,
                           item: Ptr<ast::Item>) -> Ptr<ast::Item> {
    match metaitem.node {
        ast::MetaWord(..) => {
            let change_ident_str = get_name(token::intern("change_ident_to"));
            match item.node {
                ast::ItemImpl(ref generics, ref maybe_trait, ref ty_ptr, ref impl_items) => {
                    let mut new_impl_items = Vec::with_capacity(impl_items.len());
                    for impl_item in impl_items.iter() {
                        match *impl_item {
                            ast::MethodImplItem(ref ptr) => {
                                let ref old_method = *ptr;
                                let mut new_attrs: Vec<ast::Attribute> = Vec::with_capacity(old_method.attrs.len());
                                match old_method.node {
                                    ast::MethDecl(old_ident, ref generics, abi, ref explicit_self,
                                                  fn_style, ref decl, ref block, vis) => {
                                        let mut new_name = old_ident.name;
                                        for attr in old_method.attrs.iter() {
                                            match attr.node.value.node {
                                                ast::MetaList(ref interned_str, _) => {
                                                    if change_ident_str == *interned_str {
                                                        new_name = token::intern(ident_from_meta_item(context,
                                                                                                      span,
                                                                                                      &*attr.node.value).as_slice());
                                                    }
                                                    else {
                                                        new_attrs.push(attr.clone());
                                                    }
                                                }
                                                _ => new_attrs.push(attr.clone())
                                            }
                                        }
                                        let new_method = Ptr(ast::Method {
                                            attrs: new_attrs,
                                            id: old_method.id.clone(),
                                            span: old_method.span.clone(),
                                            node: ast::MethDecl(ast::Ident {
                                                                    name: new_name,
                                                                    ctxt: old_ident.ctxt.clone()
                                                                  },
                                                                  generics.clone(), abi, explicit_self.clone(),
                                                                  fn_style, decl.clone(), block.clone(), vis)
                                        });
                                        let new_impl_item = ast::MethodImplItem(new_method);
                                        new_impl_items.push(new_impl_item);
                                    }
                                    //FIXME: This case should be handled eventually.
                                    ast::MethMac(..) => fail!("Handling of macros in method position not yet implemented by “method_modifiers”.")
                                }
                            }
                            ref some_impl_item @ _ => new_impl_items.push(some_impl_item.clone())
                        }
                    }
                    Ptr(ast::Item {
                        ident: item.ident,
                        attrs: item.attrs.clone(),
                        id:    item.id,
                        node:  ast::ItemImpl(generics.clone(), maybe_trait.clone(), ty_ptr.clone(), new_impl_items),
                        vis:   item.vis,
                        span:  item.span
                    })
                }
                _ => {
                    context.span_err(span, "“method_modifiers” expects an item impl.");
                    item.clone()
                }
            }
        }
        _ => {
            context.span_err(span, "“method_modifiers” is used without arguments.");
            item
        }
    }
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
                          item: Ptr<ast::Item>) -> Ptr<ast::Item> {
    let new_name = ident_from_meta_item(context, span, metaitem);
    match item.node {
        ast::ItemFn(..) => {
            let new_item = Ptr(ast::Item {
                ident: ast::Ident {
                           name: token::intern(new_name.as_slice()),
                           ctxt: ast::EMPTY_CTXT
                       },
                attrs: item.attrs.clone(),
                id: item.id.clone(),
                node: item.node.clone(),
                vis: item.vis.clone(),
                span: item.span.clone()
            });
            new_item
        }
        _ => {
            context.span_err(span, "“change_ident_to” unimplemented for this kind of item.");
            item
        }
    }
}


//These two conversion functions are intentionally simple and make assumptions
//suited for their specific use cases in this file.
fn snake_to_camel(xs: &[Ascii]) -> String {
    let mut i = xs.iter();
    let mut result = String::with_capacity(i.len());
    match i.next() {
        None => (),
        Some(c) => {
            result.push_char(c.to_uppercase().to_char());
            let mut last_was_underscore = false;
            for x in i {
                if *x == unsafe { '_'.to_ascii_nocheck() } {
                    last_was_underscore = true
                }
                else {
                    if last_was_underscore {
                        last_was_underscore = false;
                        result.push_char(x.to_uppercase().to_char())
                    }
                    else {
                        result.push_char(x.to_char())
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
            result.push_char(c.to_lowercase().to_char());
            for x in i {
                if x.is_uppercase() {
                    result.push_char('_');
                    result.push_char(x.to_lowercase().to_char())
                }
                else {
                    result.push_char(x.to_char())
                }
            }
        }
    }
    result
}

