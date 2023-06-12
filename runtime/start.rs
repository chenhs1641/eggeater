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
        eprintln!("index out of bound, {}", code2 / 2);
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
    sn_print(i, Vec::new());
    println!();
}

#[export_name = "\x01snek_equal"]
fn equal_value(i1:i64, i2:i64) -> i64 {
    if sn_equal(i1, i2, Vec::new(), Vec::new()) {
        return 7;
    } else {
        return 3;
    }
}

fn sn_print(i:i64, env:Vec::<i64>) {
    if i % 2 == 0 {
        print!("{}", i / 2);
    } else if i == 7 {
        print!("true");
    } else if i == 3 {
        print!("false");
    } else if i == 1 {
        print!("nil");
    } else if i & 3 == 1 {
        if env.contains(&i) {
            print!("...");
            return;
        }
        let mut new_env = env.clone();
        new_env.push(i);
        print!{"(tuple"};
        let addr: *const u64 = (i - 1) as *const u64;
        let len_tp = unsafe{ *addr };
        // println!("{}", i - 1);
        // println!("{}", len_tp);
        for j in 1..=len_tp {
            print!(" ");
            sn_print(unsafe{ *addr.offset(j as isize)} as i64, new_env.clone());
        }
        print!{")"}
    } else {
        println!("Unknown:{}", i);
    }
}

fn sn_equal(i1:i64, i2:i64, env1:Vec::<i64>, env2:Vec::<i64>) -> bool {
    let mut new_env1 = env1.clone();
    let mut new_env2 = env2.clone();
    new_env1.push(i1);
    new_env2.push(i2);
    if i1 % 2 == 0 || i1 == 7 || i1 == 3 || i1 == 1 {
        return i1 == i2
    } else if i1 & 3 == 1 {
        if i2 & 3 != 1 {
            return false;
        }
        if env1.contains(&i1) && env2.contains(&i2) {
            return true;
        }
        let mut new_env1 = env1.clone();
        let mut new_env2 = env2.clone();
        new_env1.push(i1);
        new_env2.push(i2);
        let addr1: *const u64 = (i1 - 1) as *const u64;
        let addr2: *const u64 = (i2 - 1) as *const u64;
        let len_tp1 = unsafe{ *addr1 };
        let len_tp2 = unsafe{ *addr2 };
        if len_tp1 != len_tp2 {
            return false;
        }
        for j in 1..=len_tp1 {
            if !sn_equal(unsafe{ *addr1.offset(j as isize)} as i64, unsafe{ *addr2.offset(j as isize)} as i64, new_env1.clone(), new_env2.clone()) {
                return false;
            }
        }
        return true;
    } else {
        panic!("Unknown:{}", i1);
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
