fn fact(n: u64) -> u64 {
    let mut tmp = 1;
    for i in 2..(n+1) {
        tmp *= i;
    }
    tmp
}

fn nth_perm(n: usize, index: usize) -> Vec<usize> {
    let mut result: Vec<usize> = Vec::new();
    let mut tmp = index;
    for i in 0..n {
        result.push(i);
    }

    for i in 0..(n-1) {
        let pos = tmp % (n-i);
        if pos > 0 {
            let buf = result[i];
            result[i] = result[i+pos];
            result[i+pos] = buf;
        }
        tmp = tmp / (n-i);
    }
    result
}

fn test_queens(xpos: &Vec<usize>) -> bool {
    let size = xpos.len();
    assert!(size < 32, "too many queens!");

    let mut diagsa: u64 = 0;
    let mut diagsb: u64 = 0;
    for i in 0..size {
        let diaga: u64 = 1 << (xpos[i] + i);
        let diagb: u64 = 1 << (size + xpos[i] - i);
        if ((diagsa & diaga) > 0) ||
            ((diagsb & diagb) > 0) {
            return false;
        } else {
            diagsa |= diaga;
            diagsb |= diagb;
        }
    }
    true
}

fn print_queens(xpos: &Vec<usize>) {
    let size = xpos.len();
    for _ in 0..size {
        print!("__");
    }
    print!("_");
    println!("");

    for y in 0..size {
        print!("|");
        for x in 0..size {
            if x == xpos[y] {
                print!("Q|");
            } else {
                print!("_|");
            }
        }
        println!("");
    }
    println!("===========");
}

fn main() {
    let size: usize = 8;
    for i in 0..fact(size as u64) {
        let perm = nth_perm(size, i as usize);
        if test_queens(&perm) {
            print_queens(&perm);
        }
    }
}
