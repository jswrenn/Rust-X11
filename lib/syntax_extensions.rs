#![crate_name = "syntax_extensions"]
#![crate_type = "dylib"]
#![feature(plugin_registrar, phase)]

extern crate syntax;
extern crate rustc;

use syntax::ast;
use syntax::codemap::Span;
use syntax::parse::token;
use syntax::ext::base::{ItemModifier, ExtCtxt};
use syntax::ext::build::AstBuilder;
use std::gc::{Gc, GC};

#[plugin_registrar]
pub fn registrar(reg: &mut rustc::plugin::Registry) {
  reg.register_syntax_extension(token::intern("make_predicate"),
                                ItemModifier(make_predicate_expand));
}

fn make_predicate_expand(context: &mut ExtCtxt, span: Span, metaitem: Gc<ast::MetaItem>,
                         item: Gc<ast::Item>) -> Gc<ast::Item> {
    println!("{}", metaitem);
    println!("{}", item);
    println!("testing.... testing");
    match item.node {
        ast::ItemFn(..) => {
            let old_name = item.ident.as_str();
            let new_name = format!("is_{}", old_name);
            let new_item = box(GC) ast::Item {
                ident: ast::Ident {
                           name: token::intern(new_name.as_slice()),
                           ctxt: ast::EMPTY_CTXT
                       },
                attrs: item.attrs.clone(),
                id: item.id.clone(),
                node: item.node.clone(),
                vis: item.vis.clone(),
                span: item.span.clone()
            };
            new_item
        }
        _ => {
            context.span_err(span, "make_predicate expects a function.");
            item
        }
    }
}
