#![crate_name = "new_type"]
#![crate_type = "dylib"]
#![feature(if_let, plugin_registrar, phase, quote)]

extern crate syntax;
extern crate rustc;

use syntax::ast;
use syntax::codemap::Span;
use syntax::parse::token;
use syntax::ext::base::{ExtCtxt, Decorator};
use syntax::ext::quote::rt::{ExtParseUtils, ToTokens, ToSource};
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
    let val_method_str = format!("as_{}", old_type_str);
    let old_type_name = context.parse_tts(old_type_str);
    let val_method_name = context.parse_tts(val_method_str.clone());
    let ref_method_name = context.parse_tts(format!("{}_ref", val_method_str));
    let val_method_comment_str = format!("Returns the underlying `{old}` in the `{new}`.", old=old_type_source, new=new_type_source);
    let val_method_comment_tokens = val_method_comment_str.as_slice().to_tokens(context);
    let val_method_comment = quote_tokens!(context, #[doc=$val_method_comment_tokens]);
    let ref_method_comment_str = format!("Returns a reference to the underlying `{old}` in the `{new}`.", old=old_type_source, new=new_type_source);
    let ref_method_comment_tokens = ref_method_comment_str.as_slice().to_tokens(context);
    let ref_method_comment = quote_tokens!(context, #[doc=$ref_method_comment_tokens]);
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

fn type_to_ident_str(context: &mut ExtCtxt, span: Span, type_: Ptr<ast::Ty>, format: fn(&[Ascii]) -> String) -> Option<String> {
    let mut sub_type = &type_;
    let mut result = String::new();
    loop {
        match sub_type.node {
            ast::TyNil => {
                result.push_str("Nil");
                break
            }
            ast::TyBot => {
                result.push_str("Bot");
                break
            }
            ast::TyVec(ref ty) => {
                result.push_str("Slice");
                sub_type = ty
            }
            ast::TyFixedLengthVec(ref ty, ref expr) => {
                let expr_str = expr.to_source();
                result.push_str("Array");
                result.push_str(expr_str.as_slice());
                sub_type = ty
            }
            ast::TyPtr(ref mut_ty) => {
                let ast::MutTy {
                    ty: ref ty,
                    mutbl: mutbl
                } = *mut_ty;
                result.push_str("Ptr");
                if let ast::MutMutable = mutbl { result.push_str("Mut") }
                sub_type = ty
            }
            ast::TyRptr(_, ref mut_ty) => {
                let ast::MutTy {
                    ty: ref ty,
                    mutbl: mutbl
                } = *mut_ty;
                result.push_str("Ref");
                if let ast::MutMutable = mutbl { result.push_str("Mut") }
                sub_type = ty
            }
            ast::TyTup(ref vec) => {
                result.push_str("Tup");
                for ty in vec.iter() {
                    if let Some(str) = type_to_ident_str(context, span, ty.clone(), format) {
                        result.push_str(str.as_slice())
                    }
                }
                break
            }
            ast::TyPath(ref path, _, _) => {
                for segment in path.segments.iter() {
                    result.push_str(segment.identifier.as_str())
                }
                break
            }
            ast::TyParen(ref ty) => sub_type = ty,
            _ => {
                context.span_err(span, "Unsupported type for automatic conversion to identifier.");
                println!("{}", sub_type.node);
                return None
            }
        }
    }
    Some(format(unsafe{result.to_ascii_nocheck()}))
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
