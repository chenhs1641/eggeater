use std::env;

#[link(name = "our_code")]
extern "C" {
    // The \x01 here is an undocumented feature of LLVM that ensures
    // it does not add an underscore in front of the name.
    // Courtesy of Max New (https://maxsnew.com/teaching/eecs-483-fa22/hw_adder_assignment.html)
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here(input: i64) -> i64;
}

#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode: i64) {
    // TODO: print error message according to writeup
    if errcode == 1 {
        eprintln!("invalid argument");
    } else if errcode == 2 {
        eprintln!("overflow");
    } else {
        eprintln!("an error ocurred {errcode}");
    }
    std::process::exit(1);
}

fn parse_input(input: &str) -> i64 {
    // TODO: parse the input string into internal value representation
    // 0
    if input == "false" {
        1
    }
    else if input == "true" {
        3
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
    if i % 2 == 0 {
        println!("{}", i / 2);
    } else if i == 3 {
        println!("true");
    } else if i == 1 {
        println!("false");
    } else {
        println!("Unknown:{}", i);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() == 2 { &args[1] } else { "false" };
    let input = parse_input(&input);

    let i: i64 = unsafe { our_code_starts_here(input) };
    print_value(i);
}
