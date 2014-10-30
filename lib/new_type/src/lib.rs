#![feature(macro_rules)]

#[macro_export]
macro_rules! new_type(
    ($(#[$ATTRIBUTES:meta])*
     type $A:ident = $B:ty) => (
         $(#[$ATTRIBUTES])*
         pub struct $A {
             data: $B
         }

         impl $A {
             #[inline]
             pub fn new(data: $B) -> $A {
                 $A { data: data }
             }
             ///Returns the underlying data in `self`.
             #[inline]
             pub fn as_raw_data(&self) -> $B {
                 self.data
             }

             ///Returns a reference to the underlying data in `self`.
             #[inline]
             pub fn as_raw_ref(&self) -> &$B {
                 &self.data
             }
         }
    );
)
