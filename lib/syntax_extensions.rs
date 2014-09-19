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
                        let ast::MethodImplItem(ref ptr) = *impl_item;
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
                for itm in vec.iter() {
                    match itm.node {
                        ast::MetaWord(ref interned_str) => new_name.push_str(interned_str.get()),
                        _ => {
                            context.span_err(span,
                                             "“change_ident_to” expects plain identifier expressions in its arguments. (Or arguments which become a plain identifier expression after concatenation.)");
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
