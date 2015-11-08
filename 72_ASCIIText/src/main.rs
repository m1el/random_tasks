use std::fs::File;
use std::io::Read;
use std::char;
use std::collections::{HashMap};
use std::string::String;
use std::env;

#[macro_use]
extern crate bitflags;

fn slurp(s: &String) -> Result<Vec<u8>, std::io::Error> {
    let mut file = try!(File::open(s));
    let mut buffer: Vec<u8> = Vec::new();
    let _ = try!(file.read_to_end(&mut buffer));
    Ok(buffer)
}

struct FigletRow {
    start: Option<usize>,
    end: Option<usize>,
    text: Vec<char>,
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
    fn from_string(row: String) -> FigletRow {
        let mut output: Vec<char> = Vec::new();
        let mut start = None;
        let mut end = None;
        let it = row.chars();
        //assert!(it.next() == Some(' '));
        for (i, c) in it.enumerate() {
            if c == '@' {
                continue;
            }
            output.push(c);
            let whitespace = c == ' ';

            if !whitespace {
                if start.is_none() {
                    start = Some(i);
                }
                end = Some(i);
            }
        }
        FigletRow { start: start, end: end, text: output }
    }
}

impl FigletChar {
    fn dump(&self) {
        println!("width: {}", self.width);
        for r in self.data.iter() {
            let text = r.text.iter().map(|c|*c).collect::<String>();
            println!("`{}`, start: {}, end: {}",
                     text, r.start.unwrap_or(999), r.end.unwrap_or(999));
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
            .split(|c| c == '\r'|| c == '\n');

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

        let mut it = it.skip(params.comment_lines as usize);
        let mut c = 0x20;
        let mut has_index = false;
        let mut chars: HashMap<char, FigletChar> = HashMap::new();
        loop {
            let first = match it.next() {
                None => break,
                Some(l) => l.to_string(),
            };
            if first.len() == 0 {
                break;
            }

            let mut char_lines: Vec<FigletRow> = Vec::new();
            let mut rest = params.char_height;
            let chr: char;
            if *first.as_bytes().last().unwrap() != b'@' {
                // TODO: read char number
                has_index = true;
                chr = '`';
            } else {
                if has_index {
                    println!("{}", first);
                    return Err("unexpected char without index");
                }
                char_lines.push(FigletRow::from_string(first));
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
                let row = FigletRow::from_string(line);
                if row.text.len() > width {
                    width = row.text.len();
                }
                char_lines.push(row);
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

    fn kern(&self, lch: &FigletChar, rch: &FigletChar, mode: SmushFlags) -> usize {
        let hb = self.params.hardblank;
        let mut min_overlap =
            if lch.width > rch.width { rch.width }
            else { lch.width };

        for i in 0..self.params.char_height {
            let lrow = &lch.data[i];
            let rrow = &rch.data[i];
            if lrow.end.is_none() || rrow.start.is_none() {
                continue;
            }

            let end = lrow.end.unwrap();
            let start = rrow.start.unwrap();

            let lcch = lrow.text[end];
            let rcch = rrow.text[start];
            let overlap = lch.width - end + start;
            if let Some(_) = smushem(lcch, rcch, hb, mode) {
                if min_overlap > overlap {
                    min_overlap = overlap;
                }
            } else {
                if min_overlap > overlap - 1 {
                    min_overlap = overlap - 1;
                }
            }
        }

        return min_overlap;
    }

    fn build_text(&self, s: &String, mode: SmushFlags) -> Vec<String> {
        let hb = self.params.hardblank;
        let height = self.params.char_height;
        let mut result: Vec<Vec<char>> = Vec::new();
        for _ in 0..self.params.char_height {
            result.push(Vec::new());
        }

        let mut lastchar: Option<&FigletChar> = None;

        for c in s.chars() {
            let chr = &self.chars[&c];
            let overlap;
            if let Some(lastchar) = lastchar {
                overlap = self.kern(lastchar, &chr, mode);
                for y in 0..self.params.char_height {
                    let pos = result[y].len() - overlap;
                    for i in 0..overlap {
                        let lch = result[y][pos+i];
                        let rch = chr.data[y].text[i];
                        let pp = smushem(lch, rch, hb, mode);
                        if pp.is_none() {
                            println!("smush failed: {}, {}", lch, rch);
                        }
                        result[y][pos+i] = pp.unwrap_or('#');
                    }
                }
            } else {
                overlap = 0;
            }

            for y in 0..height {
                for i in overlap..chr.width {
                    let row = &chr.data[y];
                    let fchar = row.text[i];
                    result[y].push(fchar);
                }
            }

            lastchar = Some(&chr);
        }

        return result.iter()
            .map(|cs| cs.iter()
                 .map(|c|if *c == hb { ' ' } else { *c })
                 .collect::<String>())
            .collect();
    }

    fn print_text(&self, s: &String, mode: SmushFlags) {
        let strs = self.build_text(s, mode);
        for l in strs {
            println!("{}", l);
        }
    }
}

bitflags! {
    flags SmushFlags: u32 {
        const SM_EQUAL = 0b000001,
        const SM_UNDERSCORE = 0b000010,
        const SM_HIERARCHY = 0b000100,
        const SM_BRACKETS = 0b001000,
        const SM_CROSS = 0b010000,
        const SM_HARDBLANK = 0b010000,
        const SM_ALL = 0b111111,
    }
}

/****************************************************************************
  smushem

  Given 2 characters, attempts to smush them into 1, according to
  smushmode.  Returns smushed character or '\0' if no smushing can be
  done.

  smushmode values are sum of following (all values smush blanks):
   SM_EQUAL: Smush equal chars (not hardblanks)
   SM_UNDERSCORE: Smush '_' with any char in hierarchy below
   SM_HIERARCHY: hierarchy: "|", "/\", "[]", "{}", "()", "<>"
       Each class in hier. can be replaced by later class.
   SM_BRACKETS: [ + ] -> |, { + } -> |, ( + ) -> |
   SM_CROSS: / + \ -> X, > + < -> X (only in that order)
   SM_HARDBLANK: hardblank + hardblank -> hardblank
****************************************************************************/

fn smushem(lch: char, rch: char, hardblank: char, mode: SmushFlags) -> Option<char> {
    fn get_hy(c: char) -> Option<u64> {
        match c {
            '|' => Some(0),
            '/' | '\\' => Some(1),
            '[' | ']' => Some(2),
            '{' | '}' => Some(3),
            '(' | ')' => Some(4),
            '<' | '>' => Some(5),
            _ => None,
        }
    }

    if lch == ' ' {
        return Some(rch);
    }
    if rch == ' ' {
        return Some(lch);
    }

    if mode.intersects(SM_EQUAL) && lch == rch {
        return Some(lch);
    }

    if mode.intersects(SM_UNDERSCORE) {
        if lch == '_' && get_hy(rch).is_some() {
            return Some(rch);
        }
        if rch == '_' && get_hy(lch).is_some() {
            return Some(lch);
        }
    }

    if mode.intersects(SM_HIERARCHY) {
        if let (Some(lhy), Some(rhy)) =
                 (get_hy(lch), get_hy(rch)) {
            if lhy > rhy { return Some(lch); }
            if rhy > lhy { return Some(rch); }
        }
    }

    if mode.intersects(SM_BRACKETS) {
        match (lch, rch) {
            ('[', ']') | (']', '[') => return Some('|'),
            ('{', '}') | ('}', '{') => return Some('|'),
            ('(', ')') | (')', '(') => return Some('|'),
            _ => (),
        }
    }

    if mode.intersects(SM_CROSS) {
        match (lch, rch) {
            ('/', '\\') => return Some('|'),
            ('\\', '/') => return Some('Y'),
            ('>', '<') => return Some('X'),
            _ => (),
        }
    }

    if mode.intersects(SM_HARDBLANK) {
        if lch == hardblank && rch == hardblank {
            return Some(hardblank);
        }
    }

    return None;
}

fn main() {
    let mut arguments = env::args().skip(1);
    let fname =
        if let Some(f) = arguments.next() {
            f
        } else {
            println!("not enough arguments, font name is required");
            return;
        };

    let buf = match slurp(&fname) {
        Ok(b) => b,
        Err(x) => panic!(x),
    };

    let font = match FigletFont::from_string(buf) {
        Ok(f) => f,
        Err(x) => panic!(x),
    };

    if false {
        font.dump();
        font.chars[&' '].dump();
    }

    let mut printed = false;
    for arg in arguments {
        printed = true;
        font.print_text(&arg, SM_ALL);
    }

    if !printed {
        font.print_text(&"Hello, world!".to_string(), SM_ALL);
    }
}
