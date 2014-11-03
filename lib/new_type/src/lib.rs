#![crate_name = "new_type"]
#![crate_type = "dylib"]
#![feature(if_let, plugin_registrar, phase, quote)]

extern crate syntax;
extern crate rustc;

use syntax::ast;
use syntax::codemap::Span;
use syntax::parse::token;
use syntax::ext::base::{ExtCtxt, Decorator};
use syntax::ext::quote::rt::{ToTokens, ToSource};
use syntax::ptr::P as Ptr;

#[plugin_registrar]
pub fn registrar(reg: &mut rustc::plugin::Registry) {
  reg.register_syntax_extension(token::intern("new_type"),
                                Decorator(box expand_new_type));
}

fn expand_new_type(context: &mut ExtCtxt, span: Span,
                   metaitem: &ast::MetaItem, item: &ast::Item, push: |Ptr<ast::Item>|) {
    match metaitem.node {
        ast::MetaWord(..) => (), //intentionally empty
        _ => {
            context.span_err(span, "“new_type” is used without arguments.");
            return;
        }
    }
    let (structdef_ptr, generics) = match item.node {
        ast::ItemStruct(ref structdef_ptr, ref generics) => (structdef_ptr, generics),
        _ => {
            context.span_err(span, "“new_type” is used on struct definitions only.");
            return;
        }
    };
    if generics.is_type_parameterized() {
        context.span_err(span, "“new_type” is not used with type parameterized structs.");
        return;
    }
    if let Some(..) = structdef_ptr.ctor_id {
        context.span_err(span, "“new_type” is not used with tuple- or enum-like structs.");
        return;
    }
    if structdef_ptr.fields.len() != 1 {
        context.span_err(span, "“new_type” is used on structs with exactly one field.");
        return;
    }
    let ref struct_field = structdef_ptr.fields[0].node;
    let identifier = match struct_field.kind {
        ast::NamedField(identifier, ast::Inherited) => identifier,
        _ => {
            context.span_err(span, "“new_type” is used only on structs with exactly one named private field.");
            return;
        }
    };
    let new_type = item.ident;
    let ref old_type = struct_field.ty;
    let new_type_source = new_type.to_source();
    let old_type_source = old_type.to_source();
    let old_type_str = match type_to_ident_str(context, span, old_type.clone(), camel_to_snake) {
        Some(str) => str,
        None => return
    };
    let old_type_name = token::str_to_ident(old_type_str.as_slice());

    let val_method_name = token::str_to_ident(format!("as_{}", old_type_str).as_slice());
    let val_method_comment = make_comment(context, format!("Returns the underlying `{old}` in the `{new}`.", old=old_type_source, new=new_type_source));

    let ref_method_name = token::str_to_ident(format!("as_{}_ref", old_type_str).as_slice());
    let ref_method_comment = make_comment(context, format!("Returns a reference to the underlying `{old}` in the `{new}`.", old=old_type_source, new=new_type_source));

    let maybe_item_impl = quote_item!(context,
                                      ///Automatically generated methods from
                                      ///`new_type` syntax extension attribute.
                                      impl $new_type {
                                          #[inline]
                                          pub fn new($old_type_name: $old_type) -> $new_type {
                                              $new_type { $identifier: $old_type_name }
                                          }
                                          $val_method_comment
                                          #[inline]
                                          pub fn $val_method_name(&self) -> $old_type {
                                              self.$identifier
                                          }
                                          $ref_method_comment
                                          #[inline]
                                          pub fn $ref_method_name(&self) -> &$old_type {
                                              &self.$identifier
                                          }
                                      }
                                     );
    push(maybe_item_impl.expect("Entered unexpected branch in expansion of “new_type” attribute."))
}

fn make_comment(context: &mut ExtCtxt, comment: String) -> Vec<ast::TokenTree> {
    let comment_tokens = comment.as_slice().to_tokens(context);
    quote_tokens!(context, #[doc=$comment_tokens])
}

fn type_to_ident_str(context: &mut ExtCtxt, span: Span, type_: Ptr<ast::Ty>, format: fn(&[Ascii]) -> String) -> Option<String> {
    use std::str::{MaybeOwned, Slice, Owned};

    let mut sub_type = &type_;
    let mut type_names: Vec<MaybeOwned> = Vec::new();
    loop {
        match sub_type.node {
            ast::TyNil => {
                type_names.push(Slice("nil"));
                break
            }
            ast::TyBot => {
                type_names.push(Slice("bot"));
                break
            }
            ast::TyVec(ref ty) => {
                type_names.push(Slice("slice"));
                sub_type = ty
            }
            ast::TyFixedLengthVec(ref ty, ref expr) => {
                let expr_str = expr.to_source();
                type_names.push(Slice("array"));
                type_names.push(Owned(expr_str));
                sub_type = ty
            }
            ast::TyPtr(ref mut_ty) => {
                let ast::MutTy {
                    ref ty,
                    mutbl
                } = *mut_ty;
                type_names.push(Slice("ptr"));
                if let ast::MutMutable = mutbl { type_names.push(Slice("mut")) }
                sub_type = ty
            }
            ast::TyRptr(_, ref mut_ty) => {
                let ast::MutTy {
                    ref ty,
                    mutbl
                } = *mut_ty;
                type_names.push(Slice("ref"));
                if let ast::MutMutable = mutbl { type_names.push(Slice("mut")) }
                sub_type = ty
            }
            ast::TyTup(ref vec) => {
                type_names.push(Slice("tup"));
                for ty in vec.iter() {
                    if let Some(str) = type_to_ident_str(context, span, ty.clone(), format) {
                        type_names.push(Owned(str))
                    }
                }
                break
            }
            ast::TyPath(ref path, _, _) => {
                if path.global { type_names.push(Slice("")) }
                for segment in path.segments.iter() {
                    type_names.push(Owned(format(unsafe{segment.identifier.as_str().to_ascii_nocheck()})))
                }
                break
            }
            ast::TyParen(ref ty) => sub_type = ty,
            _ => {
                context.span_err(span, "Unsupported type for automatic conversion to identifier.");
                return None
            }
        }
    }
    Some(format(unsafe{type_names.connect("_").to_ascii_nocheck()}))
}

//FIXME: Temporarily pasted version from other file.
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
