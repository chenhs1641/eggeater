use std::env;

#[link(name = "our_code")]
extern "C" {
    // The \x01 here is an undocumented feature of LLVM that ensures
    // it does not add an underscore in front of the name.
    // Courtesy of Max New (https://maxsnew.com/teaching/eecs-483-fa22/hw_adder_assignment.html)
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here(input: i64, r15: *mut u64) -> i64;
}

#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode: i64, code2: i64) {
    // TODO: print error message according to writeup
    if errcode == 1 {
        eprintln!("invalid argument");
    } else if errcode == 2 {
        eprintln!("overflow");
    } else if errcode == 3 {
        eprintln!("index out of bound, {}", code2);
    } else if errcode == 4 {
        eprintln!("try to index of nil");
    } else {
        eprintln!("an error ocurred {errcode}");
    }
    std::process::exit(1);
}

fn parse_input(input: &str) -> i64 {
    // TODO: parse the input string into internal value representation
    // 0
    if input == "nil" {
        1
    }
    else if input == "false" {
        3
    }
    else if input == "true" {
        7
    }
    else {
        let num = match input.parse::<i64>() {
            Ok(n) => n,
            Err(_) => panic!("Invalid"),
        };
        if num < -4611686018427387904 || num > 4611686018427387903 {
            panic!("Invalid");
        }
        num * 2
    }
}

#[export_name = "\x01snek_print"]
fn print_value(i:i64) {
    sn_print(i);
    println!();
}

fn sn_print(i:i64) {
    if i % 2 == 0 {
        print!("{}", i / 2);
    } else if i == 7 {
        print!("true");
    } else if i == 3 {
        print!("false");
    } else if i == 1 {
        print!("nil");
    } else if i & 3 == 1 {
        print!{"(tuple"};
        let addr: *const u64 = (i - 1) as *const u64;
        let len_tp = unsafe{ *addr };
        // println!("{}", i - 1);
        // println!("{}", len_tp);
        for j in 1..=len_tp {
            print!(" ");
            sn_print(unsafe{ *addr.offset(j as isize)} as i64);
        }
        print!{")"}
    } else {
        println!("Unknown:{}", i);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() == 2 { &args[1] } else { "false" };
    let input = parse_input(&input);
    let mut memory = Vec::<u64>::with_capacity(100000);
    let buffer: *mut u64 = memory.as_mut_ptr();
    // println!("{}", buffer as u64);
    let i: i64 = unsafe { our_code_starts_here(input, buffer) };
    print_value(i);
}
