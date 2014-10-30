#![feature(macro_rules)]

#[macro_export]
macro_rules! refined_type(
    ($(use $USING_IDS:ident);*;
     $(#[$ATTRIBUTES:meta])*
     refined $A:ident = $B:ident where
         |$ID:ident:$C:ty| -> $($PROPERTIES:ident <=> $PREDICATES:expr),+) => (
             #[change_ident_to(snake_case($A))]
             pub mod $A {
                 $(use $USING_IDS);*;

                 $(#[$ATTRIBUTES])*
                 pub struct $A {
                     data: $B
                 }

                 pub enum InvariantStatus {
                     Valid,
                     Invalid(InvariantError)
                 }

                 #[inner_attributes]
                 pub enum InvariantError {
                     $(
                         #[change_ident_to(Not, CamelCase($PROPERTIES))]
                         $PROPERTIES
                      ),+
                 }

                 #[inner_attributes]
                 impl $A {

                     $(  #[change_ident_to(is_, snake_case($PROPERTIES))]
                         pub fn $PROPERTIES(&self) -> bool {
                             let $ID: $C = self.data;
                             $PREDICATES
                         }
                      )+
                     ///Determine whether or not `self` respects the invariant.
                     pub fn meets_invariant(&self) -> bool {
                         let $ID: $C = self.data;
                         and_all!($($PREDICATES)+)
                     }
                     ///Determine whether or not `self` respects the invariant,
                     ///and if not, return the first error found.
                     pub fn test_invariant(&self) -> InvariantStatus {
                         let $ID: $C = self.data;
                         $(
                             if !$PREDICATES {
                                 return Invalid(CamelCase!(concat_idents!(not_, $PROPERTIES)))
                             }
                         )+
                             Valid
                     }
                     ///Construct by assuming `data` satifies the invariant.
                     pub unsafe fn assume(data: $B) -> $A {
                         let val = $A { data: data };
                         //Assuming should be faster but not difficult to debug.
                         debug_assert!(val.meets_invariant());
                         val
                     }
                     ///Optionally construct depending on whether `data` satisfies the invariant.
                     pub fn new(data: $B) -> Option<$A> {
                         let val = $A { data: data };
                         if val.meets_invariant() {
                             Some(val)
                         }
                         else {
                             None
                         }
                     }

                     ///Optionally construct depending on whether `data` satisfies the invariant,
                     ///and if not, return the first error found.
                     pub fn new_or_err(data: $B) -> Result<$A, InvariantError> {
                         let val = $A { data: data };
                         match val.test_invariant() {
                             Valid => Ok(val),
                             Invalid(error) => Err(error)
                         }
                     }

                     ///Get the underlying data to be free from invariant restrictions.
                     #[inline]
                     #[change_ident_to(as_, snake_case($B))]
                     pub fn raw(&self) -> $B { self.data }
                 }
             }
    );
)

#[macro_export]
macro_rules! and_all(
    ($X:expr) => ($X);
    ($X:expr $($XS:expr)+) => ( ($X) && ($(and_all!($XS))+));
)

