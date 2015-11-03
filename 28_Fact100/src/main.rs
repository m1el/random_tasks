extern crate bignum;
use bignum::{BigUint};

fn main() {
    let mut n = BigUint::from_u64(1);
    let mut tmp = BigUint::from_u64(0);

    for i in 1..101 {
        tmp.limbs[0] = i;
        n = n.mul(&tmp);
    }

    match n.to_base_string(10) {
        Ok(s) => { println!("{}", s); },
        Err(err) => { panic!(err); },
    };
}
