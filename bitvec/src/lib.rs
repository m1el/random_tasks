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
        let rest = n % BITS_IN_NUM;
        let limbs = n / BITS_IN_NUM +
            if rest > 0 { 1 } else { 0 };

        if n > self.size {
            self.limbs.resize(limbs, 0);
        } else {
            if limbs > 0 && rest > 0 {
                let mask_bits = (BITS_IN_NUM - rest) as u32;
                self.limbs[limbs-1] &= !0 >> mask_bits;
            }
            self.limbs.truncate(limbs);
        }
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
