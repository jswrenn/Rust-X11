#![feature(phase)]

#[phase(plugin)]
extern crate new_type;

pub type U = int;

#[deriving(Rand, Show, PartialEq, Eq)]
#[new_type]
pub struct T {
    data: U
}

#[test]
fn equality() {
    use std::rand;
    let rand = rand::random::<U>();
    let mut x: T = T::new(rand);
    let mut y: T = T::new(rand); 
    let xs: [U, ..9] =
        [rand, x.as_u(), x.generic_as(),
        x.into_u(), x.generic_into(),
        *x.as_u_ref(), *x.generic_as_ref(),
        *x.as_u_mut(), *x.generic_as_mut()];
    let ys: [U, ..9] =
        [rand, y.as_u(), y.generic_as(),
        y.into_u(), y.generic_into(),
        *y.as_u_ref(), *y.generic_as_ref(),
        *y.as_u_mut(), *y.generic_as_mut()];

    for x in xs.iter() {
        for y in ys.iter() {
            assert_eq!(x, y)
        }
    }
}

#[new_type]
pub struct A<'a,'b,'c> {
    data: int + 'a + 'b + 'c
}

#[new_type]
pub struct B<'a> {
    data: &'a str
}

#[new_type]
pub struct C {
    data: &'static str
}

#[new_type]
pub struct D<'a> {
    data: &'static str
}
