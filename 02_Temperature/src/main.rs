use std::env;

#[derive(Copy, Clone)]
enum TempScale {
    Kelvin,
    Celsius,
    Fahrenheit,
    Rankine,
}

impl TempScale {
    fn from_char(c: char) -> Option<TempScale> {
        match c {
            'K' | 'k' => Some(TempScale::Kelvin),
            'C' | 'c' => Some(TempScale::Celsius),
            'R' | 'r' => Some(TempScale::Rankine),
            'F' | 'f' => Some(TempScale::Fahrenheit),
            _ => None,
        }
    }
    fn from_string(s: &String) -> Option<TempScale> {
        match s.chars().next() {
            Some(c) => TempScale::from_char(c),
            None => None,
        }
    }
    fn to_string(&self) -> String {
        match *self {
            TempScale::Kelvin => "Kelvin",
            TempScale::Celsius => "Celsius",
            TempScale::Rankine => "Rankine",
            TempScale::Fahrenheit => "Fahrenheits",
        }.to_string()
    }
}

fn parse_source(s: &String) -> Option<(TempScale, f32)> {
    let mut s = s.clone();
    let last = match s.pop() {
        Some(c) => c,
        None => return None,
    };
    let scale = TempScale::from_char(last);
    let scale = match scale {
        Some(s) => s,
        None => {
            s.push(last);
            TempScale::Celsius
        }
    };
    let number =
        match s.parse::<f32>() {
            Ok(c) => c,
            Err(_) => return None,
        };
    return Some((scale, number));
}

fn to_kelvin(scale: TempScale, val: f32) -> f32 {
    match scale {
        TempScale::Kelvin => val,
        TempScale::Celsius => val + 273.15,
        TempScale::Fahrenheit => (val + 459.67) * 5.0 / 9.0,
        TempScale::Rankine => val * 5.0 / 9.0,
    }
}

fn from_kelvin(scale: TempScale, val: f32) -> f32 {
    match scale {
        TempScale::Kelvin => val,
        TempScale::Celsius => val - 273.15,
        TempScale::Fahrenheit => val * 9.0 / 5.0 - 459.67,
        TempScale::Rankine => val * 9.0 / 5.0,
    }
}

fn convert_temp(from: TempScale, to: TempScale, val: f32) -> f32 {
    from_kelvin(to, to_kelvin(from, val))
}

fn usage() {
    println!("usage: <temperature> <target>");
    println!("usage: <temperature><scale> <target scale>");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 2 {
        usage();
        return;
    }
    let to_scale =
        match TempScale::from_string(&args[1]) {
            Some(s) => s,
            None => {
                usage();
                return;
            }
        };

    match parse_source(&args[0]) {
        Some((src_scale, number)) => {
            println!("from {} {}",
                     number, src_scale.to_string());
            let answer = convert_temp(src_scale, to_scale, number);
            println!("to {:.2} {}",
                     answer, to_scale.to_string());
        },
        None => {
            usage();
            return;
        }
    }
}
