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
    let old_type_str = unsafe {
        camel_to_snake(old_type_source.as_slice().to_ascii_nocheck())
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

//FIXME: Temporarily pasted and edited version from other file.
//I really don't like how this turned out. Hope to refactor or use a completely different technique
//later.
fn camel_to_snake(xs: &[Ascii]) -> String {
    let mut i = xs.iter();
    let mut result = String::with_capacity(i.len());
    let mut global_path = false;
    let mut already_have_one_underscore = false;
    match i.next() {
        None => (),
        Some(c) => {
            if !c.is_alphanumeric() {
                if *c == unsafe{'&'.to_ascii_nocheck()} {
                    result.push_str("ref_");
                    already_have_one_underscore = true
                }
                else if *c == unsafe{'*'.to_ascii_nocheck()} {
                    result.push_str("ptr_");
                    already_have_one_underscore = true
                }
                else if *c == unsafe{':'.to_ascii_nocheck()} {
                    global_path = true;
                    result.push('_');
                    already_have_one_underscore = true
                }
                else if *c == unsafe{'\''.to_ascii_nocheck()} {
                    result.push_str("lftm_");
                    already_have_one_underscore = true
                }
                else {
                    result.push('_');
                    already_have_one_underscore = true;
                }
            }
            else {
                result.push(c.to_lowercase().to_char())
            }
            for x in i {
                let uppercase = x.is_uppercase();
                if uppercase && !global_path {
                    if !already_have_one_underscore { result.push('_') }
                    result.push(x.to_lowercase().to_char());
                    already_have_one_underscore = false
                }
                else if uppercase && global_path {
                    global_path = false;
                    result.push(x.to_lowercase().to_char());
                    already_have_one_underscore = false
                }
                else if !x.is_alphanumeric() {
                    if *x == unsafe{'*'.to_ascii_nocheck()} {
                        result.push_str("ptr_");
                        already_have_one_underscore = true
                    }
                    else if *x == unsafe{'&'.to_ascii_nocheck()} {
                        result.push_str("ref_");
                        already_have_one_underscore = true
                    }
                    else if *x == unsafe{'\''.to_ascii_nocheck()} {
                        result.push_str("lftm_");
                        already_have_one_underscore = true
                    }
                    else if !already_have_one_underscore {
                        result.push('_');
                        already_have_one_underscore = true
                    }
                }
                else {
                    result.push(x.to_char());
                    already_have_one_underscore = false
                }
            }
        }
    }
    result
}
