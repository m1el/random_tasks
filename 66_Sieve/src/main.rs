const BITS_IN_NUM: usize = 64;
struct BitVec {
    limbs: Vec<u64>,
    size: usize,
}

impl BitVec {
    fn new() -> BitVec {
        BitVec { limbs: Vec::new(), size: 0 }
    }

    fn resize(&mut self, n: usize) {
        let limbs = n / BITS_IN_NUM +
            if n % BITS_IN_NUM > 0 { 1 } else { 0 };

        if n > self.size {
            self.limbs.resize(limbs, 0);
        } else {
            self.limbs.truncate(limbs);
        }
        self.size = n;
    }

    fn truncate(&mut self, n: usize) {
        let limbs = n / BITS_IN_NUM +
            if n % BITS_IN_NUM > 0 { 1 } else { 0 };
        self.limbs.truncate(limbs);
        self.size = n;
    }

    fn get(&self, index: usize) -> bool {
        if index >= self.size {
            panic!("out of bounds");
        }
        let limb = index / BITS_IN_NUM;
        let bit = index % BITS_IN_NUM;
        let mask = 1 << bit;
        (mask & self.limbs[limb]) > 0
    }

    fn set(&mut self, index: usize, val: bool) {
        if index >= self.size {
            panic!("out of bounds");
        }
        let limb = index / BITS_IN_NUM;
        let bit = index % BITS_IN_NUM;
        let mask: u64 = 1 << bit;
        if val {
            self.limbs[limb] = self.limbs[limb] | mask;
        } else {
            self.limbs[limb] = self.limbs[limb] & !mask;
        }
    }
}

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
