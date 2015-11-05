use std::fs::File;
use std::io::Read;
use std::char;
use std::collections::{HashMap};

fn read_file() -> Result<Vec<u8>, std::io::Error> {
    let mut file = try!(File::open("Big.flf"));
    let mut buffer: Vec<u8> = Vec::new();
    let _ = try!(file.read_to_end(&mut buffer));
    Ok(buffer)
}

struct FigletRow {
    start: usize,
    end: usize,
    text: String,
}

struct FigletChar {
    width: usize,
    data: Vec<FigletRow>,
}

struct FigParams {
    hardblank: char,
    char_height: usize,
    maxlen: i64,
    smush: i64,
    max_width: i64,
    comment_lines: i64,
    ffright2left: i64,
    smush2: i64,
    code_range: i64,
}

struct FigletFont {
    params: FigParams,
    chars: HashMap<char, FigletChar>,
}

impl FigletRow {
    fn from_string(row: String, hardblank: char) -> FigletRow {
        let mut output: String = String::new();
        let mut start: usize = 0;
        let mut end: usize = 0;
        let it = row.chars();
        //assert!(it.next() == Some(' '));
        for (i, c) in it.enumerate() {
            if c == hardblank {
                start = if start != 0 { start } else { i };
                output.push(' ');
            } else if c == '@' {
            } else {
                end = i;
                output.push(c);
            }
        }
        FigletRow { start: start, end: end, text: output }
    }
}

impl FigletChar {
    fn dump(&self) {
        println!("width: {}", self.width);
        for r in self.data.iter() {
            println!("`{}`, start: {}, end: {}",
                     r.text, r.start, r.end);
        }
        println!("======");
    }
}

impl FigletFont {
    fn dump(&self) {
        println!("{},{},{},{},{},{},{},{},{}",
            self.params.hardblank,
            self.params.char_height,
            self.params.maxlen,
            self.params.smush,
            self.params.max_width,
            self.params.comment_lines,
            self.params.ffright2left,
            self.params.smush2,
            self.params.code_range);
    }

    fn from_string(source: Vec<u8>) -> Result<FigletFont, &'static str> {
        let source =
            match String::from_utf8(source) {
                Err(_) => return Err("invalid utf-8 file"),
                Ok(s) => s.to_string(),
            };
        let mut it = source
            .split(|c| c == '\r'|| c == '\n')
            .filter(|w| w.len() != 0);

        let header = match it.next() {
            None => return Err("empty file?"),
            Some(w) => w,
        };
        let mut header_words = header.split_whitespace();
        let mut magic = match header_words.next() {
            None => return Err("empty header?"),
            Some(w) => w.to_string(),
        };
        let hardblank = match magic.pop() {
            None => return Err("empty magic?"),
            Some(c) => c,
        };
        let mut header_params: Vec<i64> = Vec::new();
        for w in header_words {
            match w.parse::<i64>() {
                Err(_) => return Err("found a non-integer parameter in header"),
                Ok(i) => header_params.push(i),
            };
        }
        if header_params.len() < 6 {
            return Err("header does not have enough(6) parameters");
        }
        let params = FigParams {
            hardblank: hardblank,
            char_height: header_params[0] as usize,
            maxlen: header_params[1],
            smush: header_params[2],
            max_width: header_params[3],
            comment_lines: header_params[4],
            ffright2left: header_params[5],
            smush2: 0,
            code_range: *header_params.get(6).unwrap_or(&0),
        };

        let mut it = it.skip(params.comment_lines as usize - 1);
        let mut c = 0x20;
        let mut has_index = false;
        let mut chars: HashMap<char, FigletChar> = HashMap::new();
        loop {
            let first = match it.next() {
                None => break,
                Some(l) => l.to_string(),
            };
            let mut char_lines: Vec<FigletRow> = Vec::new();
            let mut rest = params.char_height;
            let chr: char;
            if first.as_bytes()[0] != b' ' {
                // TODO: read char number
                has_index = true;
                chr = '`';
            } else {
                if has_index {
                    return Err("unexpected char without index");
                }
                char_lines.push(FigletRow::from_string(first, params.hardblank));
                rest -= 1;
                chr = match char::from_u32(c) {
                    None => return Err("could not convert int to char???"),
                    Some(c) => c,
                };
                c += 1;
            }

            let mut width: usize = 0;
            for _ in 0..rest {
                let line = it.next();
                if line.is_none() {
                    return Err("not enough lines in a char");
                }
                let line = line.unwrap().to_string();
                let len = line.len();
                if len > width {
                    width = len;
                }
                char_lines.push(FigletRow::from_string(line, params.hardblank));
            }

            chars.insert(chr, FigletChar{
                width: width,
                data: char_lines,
            });
        }

        Ok(FigletFont {
            params: params,
            chars: chars,
        })
    }

    fn print_char(&self, c: char) {
        if let Some(chr) = self.chars.get(&c) {
            for row in chr.data.iter() {
                println!("{}", row.text);
            }
        } else {
            println!("nope :D");
        }
    }

    fn print_string(&self, s: &String) {
        for i in 0..self.params.char_height {
            for c in (*s).chars() {
                if let Some(chr) = self.chars.get(&c) {
                    print!("{}", chr.data[i].text);
                }
            }
            println!("");
        }
    }
}

fn main() {
    let buf = read_file();
    if let Err(e) = buf { panic!(e); }
    let font = match FigletFont::from_string(buf.unwrap()) {
        Ok(f) => f,
        Err(x) => panic!(x),
    };
    if false {
        font.dump();
        font.chars[&' '].dump();
    }
    if true {
        font.print_string(&"Thx for".to_string());
        font.print_string(&"watching!".to_string());
        font.print_string(&"And good night!".to_string());
    }
}
