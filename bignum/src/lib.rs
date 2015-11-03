#![crate_type = "lib"]
#![crate_name = "bignum"]

#![feature(asm)]

use std::str;
use std::cmp::Ordering;
use std::cmp::Ordering::{Greater,Less,Equal};

// unsigned
pub struct BigUint {
    pub limbs: Vec<u64>,
}

#[derive(PartialEq,Clone)]
pub enum IntSign {
    Pos,
    Neg,
}
// signed
pub struct BigInt {
    pub sign: IntSign,
    pub num: BigUint,
}

pub const BITS_IN_LIMB: usize = 64;
const ALPHABET: &'static str =
    "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+=";

#[inline(always)]
fn long_mul(a: u64, b: u64) -> (u64, u64) {
    let mut _mul: u64;
    let mut _overflow: u64;
    unsafe {
        asm!("mul %rdx"
            :"={rax}"(_mul), "={rdx}"(_overflow)
            :"{rax}"(a), "{rdx}"(b));
    }
    (_mul, _overflow)
}

impl BigInt {
    pub fn from_i64(n: i64) -> BigInt {
        if n >= 0 {
            BigInt {
                sign: IntSign::Pos,
                num: BigUint::from_u64(n as u64)
            }
        } else {
            BigInt {
                sign: IntSign::Neg,
                num: BigUint::from_u64((-n) as u64)
            }
        }
    }

    pub fn dump(&self) {
        match self.sign {
            IntSign::Pos => print!("+"),
            IntSign::Neg => print!("-"),
        }
        self.num.dump();
    }

    pub fn to_base_string(&self, base: u64) -> Result<String, &'static str> {
        let prefix: String = (match self.sign {
                IntSign::Pos => "",
                IntSign::Neg => "-",
            }).to_string();
        match self.num.to_base_string(base) {
            Ok(s) => Ok(prefix + &s),
            Err(e) => Err(e),
        }
    }

    pub fn clone(&self) -> BigInt {
        BigInt { sign: self.sign.clone(), num: self.num.clone() }
    }
}

impl BigUint {

    pub fn dump(&self) {
        print!("[");
        for i in 0..self.limbs.len() {
            print!("{} ", self.limbs[i]);
        }
        println!("]");
    }

    pub fn digit_limb_base(base: u64) -> (u64, u64) {
        let mut step = base;
        let mut c = 1;
        loop {
            let (tmp, carry) = long_mul(step, base);
            if carry > 0 {
                return (step, c);
            } else {
                step = tmp;
                c += 1;
            }
        }
    }

    pub fn digits(&self, base: u64) -> Result<Vec<u8>, &'static str> {
        if base < 2 || base > 256 {
            return Err("The base has to be in the range [2, 256]");
        }

        let (step_u, digits_per_limb) =
            BigUint::digit_limb_base(base);
        let step = BigUint::from_u64(step_u);

        let mut rest = self.clone();
        let mut result: Vec<u8> = Vec::new();
        while !rest.is_zero() {
            let (quot, rem) = rest.divmod(&step);

            let mut limb =
                match rem.limbs.first() {
                    None => 0,
                    Some(x) => *x,
                };

            for _ in 0..digits_per_limb {
                result.push((limb % base) as u8);
                limb = limb / base;
            }

            rest = quot;
        }

        if result.len() == 0 {
            result.push(0);
        } else {
            let mut i = result.len();
            while result[i-1] == 0 && i > 1 {
                i -= 1;
            }
            if i != result.len() {
                result.truncate(i);
            }
        }

        Ok(result)
    }

    pub fn to_base_string(&self, base: u64) -> Result<String, &'static str> {
        if base < 2 || base > 64 {
            return Err("The base has to be in the range [2, 64]");
        }

        let mut digs = match self.digits(base) {
            Err(s) => return Err(s),
            Ok(digs) => digs,
        };
        digs.reverse();

        let alpha: &[u8] = ALPHABET.as_ref();
        for i in 0..digs.len() {
            digs[i] = alpha[digs[i] as usize];
        }

        match str::from_utf8(&digs) {
            Err(_) => Err("BigUint::to_base_string somehow produced invalid utf-8 bytes."),
            Ok(s) => Ok(s.to_string())
        }
    }

    pub fn from_u64(num: u64) -> BigUint {
        BigUint { limbs: vec![num] }
    }

    pub fn is_zero(&self) -> bool {
        if self.limbs.len() == 0 {
            true
        } else {
            self.limbs.len() == 1 && self.limbs[0] == 0
        }
    }

    pub fn add(&self, other: &BigUint) -> BigUint {
        let mut result = BigUint { limbs: Vec::new() };
        let (l1, l2) = (self.limbs.len(), other.limbs.len());
        let (maxlen, minlen) =
            if l1 > l2 { (l1, l2) }
            else { (l2, l1) };
        let (ref bigger, ref smaller) =
            if l1 > l2 { (&self.limbs, &other.limbs) }
            else { (&other.limbs, &self.limbs) };

        let mut carry = 0;
        for i in 0..minlen {
            let mut limb = bigger[i].wrapping_add(carry);
            carry = if limb < carry { 1 } else { 0 };
            limb = limb.wrapping_add(smaller[i]);
            carry = if carry > 0 || limb < smaller[i] { 1 } else { 0 };
            result.limbs.push(limb);
        }
        for i in minlen..maxlen {
            let limb = bigger[i].wrapping_add(carry);
            carry = if limb < carry { 1 } else { 0 };
            result.limbs.push(limb);
        }
        if carry > 0 {
            result.limbs.push(carry);
        }
        result
    }

    pub fn highest_u64_bit(n: u64) -> usize {
        let mut tmp = n;
        for i in 0..(BITS_IN_LIMB-1) {
            if tmp == 0 {
                return i;
            }
            tmp = tmp >> 1;
        }
        return BITS_IN_LIMB-1;
    }

    fn highest_bit(&self) -> usize {
        if self.limbs.len() == 0 {
            return 0;
        }

        let guaranteed_bits = (self.limbs.len() - 1) * BITS_IN_LIMB;
        match self.limbs.last() {
            None => 0,
            Some(last) => guaranteed_bits + BigUint::highest_u64_bit(*last),
        }
    }

    pub fn mul_by_limb(&mut self, n: u64) {
        let ref mut limbs = self.limbs;
        let mut carry: u64 = 0;
        for i in 0..limbs.len() {
            let (mul, over) = long_mul(limbs[i], n);
            let limb = mul.wrapping_add(carry);
            carry = if limb < carry { 1 } else { 0 };
            limbs[i] = limb;
            carry += over;
        }
        if carry > 0 {
            limbs.push(carry);
        }
    }

    pub fn bit_shift_right(&mut self, n: usize) {
        let old_len = self.limbs.len();
        if old_len == 0 {
            return;
        }

        let limb_shift = n / BITS_IN_LIMB;
        let rest_bitshift = n % BITS_IN_LIMB;
        let rest_invshift = BITS_IN_LIMB - n;

        {
            let ref mut limbs = self.limbs;
            for i in 0..(old_len-1) {
                limbs[i] = (limbs[i] >> rest_bitshift) |
                           (limbs[i+1] << rest_invshift);
            }

            let last_value = limbs[old_len-1] >> rest_bitshift;
            if last_value > 0 {
                limbs[old_len-1] = last_value;
            } else {
                let _ = limbs.pop();
            }
        }

        self.limb_shift_right(limb_shift);
    }

    pub fn bit_shift_left(&mut self, n: usize) {
        let old_len = self.limbs.len();
        let limb_shift = n / BITS_IN_LIMB;
        let rest_bitshift = (n % BITS_IN_LIMB) as u32;
        let rest_invshift = (BITS_IN_LIMB as u32) - rest_bitshift;

        {
            let ref mut limbs = self.limbs;
            let has_last = match limbs.last() {
                    None => false,
                    Some(x) => x.wrapping_shr(rest_invshift) > 0,
                };

            if rest_bitshift > 0 {
                let last_index =
                    if has_last {
                        limbs.push(0);
                        old_len
                    }
                    else { old_len - 1 };

                for i in (0..last_index).rev() {
                    limbs[i+1] = (limbs[i+1].wrapping_shl(rest_bitshift)) |
                                 (limbs[i].wrapping_shr(rest_invshift));
                }
                limbs[0] = limbs[0].wrapping_shl(rest_bitshift);
            }
        }

        if limb_shift > 0 {
            self.limb_shift_left(limb_shift);
        }
    }

    pub fn limb_shift_left(&mut self, n: usize) {
        let old_len = self.limbs.len();
        let ref mut limbs = self.limbs;
        limbs.resize(old_len + n, 0);

        for i in (0..old_len).rev() {
            limbs[i + n] = limbs[i];
        }

        for i in 0..n {
            limbs[i] = 0;
        }
    }

    pub fn limb_shift_right(&mut self, n: usize) {
        let ref mut limbs = self.limbs;
        let old_len = limbs.len();
        let new_len = if old_len < n { 0 }
                      else { old_len - n };

        for i in 0..new_len {
            limbs[i] = limbs[i+n];
        }

        limbs.truncate(new_len)
    }

    pub fn mul(&self, other: &BigUint) -> BigUint {
        let (l1, l2) = (self.limbs.len(), other.limbs.len());
        let (_maxlen, minlen) =
            if l1 > l2 { (l1, l2) }
            else { (l2, l1) };
        let (ref bigger, ref smaller) =
            if l1 > l2 { (&self.limbs, &other.limbs) }
            else { (&other.limbs, &self.limbs) };

        let mut result = BigUint::from_u64(0);
        for i in 0..minlen {
            let mut copy = BigUint { limbs: (*bigger).clone() };
            copy.limb_shift_left(i);
            copy.mul_by_limb(smaller[i]);
            result = result.add(&copy);
        }
        result
    }

    pub fn cmp(&self, other: &BigUint) -> Ordering {
        let (l1, l2) = (self.limbs.len(), other.limbs.len());
        match l1.cmp(&l2) {
            Greater => return Greater,
            Less => return Less,
            Equal => (),
        }

        for i in (0..l1).rev() {
            match self.limbs[i].cmp(&other.limbs[i]) {
                Greater => return Greater,
                Less => return Less,
                Equal => (),
            }
        }

        Equal
    }

    pub fn sub(&self, other: &BigUint) -> BigInt {
        let cmp = self.cmp(other);
        if cmp == Equal {
            return BigInt {
                sign: IntSign::Pos,
                num: BigUint::from_u64(0)
            };
        }

        let (sign, bigger, smaller) =
            match cmp {
                Greater => (IntSign::Pos, &self, &other),
                Less => (IntSign::Neg, &other, &self),
                Equal => (IntSign::Pos, &other, &self),
            };

        let (maxlen, minlen) = (bigger.limbs.len(), smaller.limbs.len());

        let mut result = bigger.limbs.clone();
        let mut carry = 0;
        for i in 0..minlen {
            let limb_a = result[i].wrapping_sub(carry);
            carry = if limb_a > result[i] { 1 } else { 0 };
            let limb = limb_a.wrapping_sub(smaller.limbs[i]);
            carry = if carry > 0 || limb > limb_a { 1 }
                    else { 0 };
            result[i] = limb;
        }

        for i in minlen..maxlen {
            let limb = result[i].wrapping_sub(carry);
            carry = if limb > result[i] { 1 } else { 0 };
            result[i] = limb;
        }

        let l = result.len();
        if l > 1 && result[l-1] == 0 {
            result.truncate(l-1);
        }

        BigInt {
            sign: sign,
            num: BigUint { limbs: result }
        }
    }

    pub fn clone(&self) -> BigUint {
        BigUint { limbs: self.limbs.clone() }
    }

    pub fn set_bit(&mut self, pos: usize) {
        let limb = pos / BITS_IN_LIMB;
        let bit = pos % BITS_IN_LIMB;
        if limb >= self.limbs.len() {
            self.limbs.resize(limb+1, 0);
        }
        self.limbs[limb] |= 1 << bit;
    }

    // returns (quotient, remainder)
    pub fn divmod(&self, divisor: &BigUint) -> (BigUint, BigUint) {
        let self_bits = self.highest_bit();
        let divisor_bits = divisor.highest_bit();
        if self_bits < divisor_bits {
            return (BigUint::from_u64(0), self.clone());
        }

        let max_shift = self_bits - divisor_bits;
        let mut tmp = self.clone();
        let mut result = BigUint::from_u64(0);
        let mut subtracting = divisor.clone();
        subtracting.bit_shift_left(max_shift);

        for i in (0..(max_shift+1)).rev() {
            // TODO: this should call in-place subtraction
            let sub = tmp.sub(&subtracting);
            if sub.sign == IntSign::Pos {
                result.set_bit(i);
                tmp = sub.num;
            }

            subtracting.bit_shift_right(1);
        }

        (result, tmp)
    }
}

