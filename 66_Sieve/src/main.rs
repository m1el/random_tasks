extern crate bitvec;
use bitvec::BitVec;

fn sieve(n: usize) -> BitVec {
    let mut result = BitVec::new();
    result.resize(n);
    for i in 2..n {
        if result.get(i) {
            continue;
        }

        let mut j = i * 2;
        while j < n {
            result.set(j, true);
            j += i;
        }
    }
    result
}

fn main () {
    let nums = 100000000;
    let bitvec = sieve(nums);
    for i in nums-100..nums {
        if !bitvec.get(i) {
            println!("{}", i);
        }
    }
}
