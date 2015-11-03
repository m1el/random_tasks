extern crate bignum;
use bignum::{BigUint};

fn main() {
    let mut a = BigUint::from_u64(1);
    let mut b = BigUint::from_u64(1);
    for _ in 0..200 {
        let next = a.add(&b);
        match a.to_base_string(10) {
            Ok(s) => { println!("{}", s); },
            Err(err) => { panic!(err); },
        };
        a = b;
        b = next;
    }
}
