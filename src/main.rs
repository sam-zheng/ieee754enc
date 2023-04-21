use clap::{Arg, ArgAction, Command};
use std::fmt::{Binary, LowerHex, Write};
use std::mem;

#[repr(transparent)]
struct F64(f64);

#[repr(transparent)]
struct F32(f32);

impl Binary for F64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let l = unsafe { mem::transmute::<f64, u64>(self.0) };
        let s = format!("{l:064b}");
        f.write_str(&s[0..1])?;
        f.write_str(" ")?;
        f.write_str(&s[1..12])?;
        f.write_str(" ")?;
        f.write_str(&s[12..])
    }
}

fn fmt_hex(n: &dyn core::any::Any, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let (s, len) = if n.is::<F64>() {
        let l = unsafe { mem::transmute::<f64, u64>(n.downcast_ref::<F64>().unwrap().0) };
        (format!("{l:016x}"), 2)
    } else if n.is::<F32>() {
        let l = unsafe { mem::transmute::<f32, u32>(n.downcast_ref::<F32>().unwrap().0) };
        (format!("{l:08x}"), 1)
    } else {
        panic!("Not a floating point numer!");
    };
    for i in 0..len {
        if i > 0 {
            f.write_char(' ')?;
        }
        f.write_str(&s[i*8..i*8 + 8])?;
    }
    std::fmt::Result::Ok(())
}

impl LowerHex for F64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_hex(self, f)
    }
}

impl LowerHex for F32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_hex(self, f)
    }
}

impl Binary for F32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let u = unsafe { mem::transmute::<f32, u32>(self.0) };
        let s = format!("{u:032b}");
        f.write_str(&s[0..1])?;
        f.write_str(" ")?;
        f.write_str(&s[1..9])?;
        f.write_str(" ")?;
        f.write_str(&s[9..])
    }
}

macro_rules! format_floats {
    ($type:ident, $n:expr, $x:ident, $prefix:ident) => {
        if $prefix {
            if $x {
                format!("{:}: {:x}", $n, $type($n))
            } else {
                format!("{:}: {:b}", $n, $type($n))
            }
        } else {
            if $x {
                format!("{:x}", $type($n))
            } else {
                format!("{:b}", $type($n))
            }
        }
    };
}

fn enc_floats(nums: Vec<&str>, hex:bool, prefix:bool) -> String {
    let mut out = String::new();
    for i in 0..nums.len() {
        if i > 0 {
            out.push('\n');
        }
        let s: &str = &nums[i].replace("_", "");
        if !s.ends_with("f32") {
            let mut t = s;
            if t.ends_with("f64") {
                t = &t[0..s.len() - 3];
            }
            let mut err = "expecting f64, but was ".to_string();
            err.push_str(s);
            let f = t.parse::<f64>().expect(&err);
            out.push_str(&format_floats!(F64, f, hex, prefix));
        } else {
            let t = &s[0..s.len() - 3];
            let mut err = "expecting f32, but was ".to_string();
            err.push_str(s);
            let f = t.parse::<f32>().expect(&err);
            out.push_str(&format_floats!(F32, f, hex, prefix));
        }
        
    }
    out
}

fn main() {
    let matches = Command::new("fbe")
        .about("Print IEEE754 binary encoding of floating point numbers")
        .arg(
            Arg::new("x")
                .short('x')
                .long("hex")
                .action(ArgAction::SetTrue)
                .help("output in hex format, default is in binary format")
                .required(false)
        )
        .arg(
            Arg::new("P")
                .short('P')
                .long("noprefix")
                .action(ArgAction::SetTrue)
                .default_value(Some("false"))
                .help("don't print the number itself as the prefix")
                .required(false)
        )
        .arg(Arg::new("numbers").action(ArgAction::Append).required(true))
        .get_matches();

    let args = matches
        .get_many::<String>("numbers")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();

    let x: bool = matches.get_flag("x");
    let np: bool = matches.get_flag("P");
    let s = enc_floats(args, x, !np);
    println!("{}", s);
    
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_fbe() {
        assert_eq!("0 00000000 00000000000000000000000", super::enc_floats(["0f32"].to_vec(), false, false));
        assert_eq!("0 01111111 00000000000000000000000", super::enc_floats(["1f32"].to_vec(), false, false));
        assert_eq!("1 01111111 00000000000000000000000", super::enc_floats(["-1f32"].to_vec(), false, false));
        assert_eq!("0 10000000 01000000000000000000000", super::enc_floats(["2.5f32"].to_vec(), false, false));
        assert_eq!("0 11111111 00000000000000000000000", super::enc_floats(["inff32"].to_vec(), false, false));
        assert_eq!("0 11111111 10000000000000000000000", super::enc_floats(["nanf32"].to_vec(), false, false));
    }

}