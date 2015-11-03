const BITS_IN_NUM: usize = 64;
pub struct BitVec {
    pub limbs: Vec<u64>,
    pub size: usize,
}

impl BitVec {
    pub fn new() -> BitVec {
        BitVec { limbs: Vec::new(), size: 0 }
    }

    pub fn resize(&mut self, n: usize) {
        let limbs = n / BITS_IN_NUM +
            if n % BITS_IN_NUM > 0 { 1 } else { 0 };

        if n > self.size {
            self.limbs.resize(limbs, 0);
        } else {
            self.limbs.truncate(limbs);
        }
        self.size = n;
    }

    pub fn truncate(&mut self, n: usize) {
        let limbs = n / BITS_IN_NUM +
            if n % BITS_IN_NUM > 0 { 1 } else { 0 };
        self.limbs.truncate(limbs);
        self.size = n;
    }

    pub fn get(&self, index: usize) -> bool {
        if index >= self.size {
            panic!("out of bounds");
        }
        let limb = index / BITS_IN_NUM;
        let bit = index % BITS_IN_NUM;
        let mask = 1 << bit;
        (mask & self.limbs[limb]) > 0
    }

    pub fn set(&mut self, index: usize, val: bool) {
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
