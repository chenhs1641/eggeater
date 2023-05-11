use std::env;
use std::fs::File;
use std::io::prelude::*;

use sexp::Atom::*;
use sexp::*;

use im::HashMap;

#[derive(Debug)]
enum Val {
  Reg(Reg),
  Imm(i64),
  RegOnset(Reg, i64),
  RegOffset(Reg, i64),
}

#[derive(Debug)]
enum Label {
  TYPEERROR,
  OVERFLOW,
  LName(String),
}

#[derive(Debug)]
enum Reg {
  RAX,
  RBX,
  RSP,
  RDI,
}

#[derive(Debug)]
enum Instr {
  IMov(Val, Val),
  IAdd(Val, Val),
  ISub(Val, Val),
  IMul(Val, Val),
  Test(Val, Val),
  Cmp(Val, Val),
  Sar(Val, Val),
  Xor(Val, Val),
  Push(Val),
  Pop(Val),
  Jmp(Label),
  Je(Label),
  Jne(Label),
  Jg(Label),
  Jl(Label),
  Jge(Label),
  Jle(Label),
  Jo(Label),
  Nothing(Label),
  Call(Label),
}

#[derive(Debug)]
enum Op1 {
  Add1,
  Sub1,
  IsNum,
  IsBool,
}

#[derive(Debug)]
enum Op2 {
  Plus,
  Minus,
  Times,
  Lt,
  Gt,
  Ge,
  Le,
  Eq,
}

#[derive(Debug)]
enum Expr {
  Number(i64),
  TRUE,
  FALSE,
  INPUT,
  Id(String),
  Let(Vec<(String, Expr)>, Box<Expr>),
  UnOp(Op1, Box<Expr>),
  BinOp(Op2, Box<Expr>, Box<Expr>),
  Set(String, Box<Expr>),
  If(Box<Expr>, Box<Expr>, Box<Expr>),
  Block(Vec<Expr>),
  Loop(Box<Expr>),
  Break(Box<Expr>),
  Funccall(String, Vec<Expr>),
}

#[derive(Debug)]
enum Statement{
  Definition(Vec<String>, Box<Expr>),
  Expression(Box<Expr>),
}

fn parse_bind(s: &Sexp) -> (String, Expr) {
  match s {
    Sexp::List(vec) => {
      match &vec[..] {
        [Sexp::Atom(S(name)), e] => (name.to_string(), parse_expr(e)),
        _ => panic!("Invalid"),
      }
    }
    _ => panic!("Invalid"),
  }
}

fn parse_expr(s: &Sexp) -> Expr {
  match s {
    Sexp::Atom(I(n)) => Expr::Number(i64::try_from(*n).unwrap()),
    Sexp::Atom(S(keyword)) if keyword == "true" => Expr::TRUE,
    Sexp::Atom(S(keyword)) if keyword == "false" => Expr::FALSE,
    Sexp::Atom(S(keyword)) if keyword == "input" => Expr::INPUT,
    Sexp::Atom(S(id)) => Expr::Id(id.to_string()),
    Sexp::List(vec) => {
      match &vec[..] {
        [Sexp::Atom(S(op)), e] if op == "add1" => Expr::UnOp(Op1::Add1, Box::new(parse_expr(e))),
        [Sexp::Atom(S(op)), e] if op == "sub1" => Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e))),
        [Sexp::Atom(S(op)), e] if op == "isnum" => Expr::UnOp(Op1::IsNum, Box::new(parse_expr(e))),
        [Sexp::Atom(S(op)), e] if op == "isbool" => Expr::UnOp(Op1::IsBool, Box::new(parse_expr(e))),
        [Sexp::Atom(S(op)), e1, e2] if op == "+" => Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
        [Sexp::Atom(S(op)), e1, e2] if op == "-" => Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
        [Sexp::Atom(S(op)), e1, e2] if op == "*" => Expr::BinOp(Op2::Times, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
        [Sexp::Atom(S(op)), e1, e2] if op == "<" => Expr::BinOp(Op2::Lt, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
        [Sexp::Atom(S(op)), e1, e2] if op == ">" => Expr::BinOp(Op2::Gt, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
        [Sexp::Atom(S(op)), e1, e2] if op == ">=" => Expr::BinOp(Op2::Ge, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
        [Sexp::Atom(S(op)), e1, e2] if op == "<=" => Expr::BinOp(Op2::Le, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
        [Sexp::Atom(S(op)), e1, e2] if op == "=" => Expr::BinOp(Op2::Eq, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
        [Sexp::Atom(S(set)), Sexp::Atom(S(name)), e2] if set == "set!" => Expr::Set(name.to_string(), Box::new(parse_expr(e2))),
        [Sexp::Atom(S(if_)), e1, e2, e3] if if_ == "if" => Expr::If(Box::new(parse_expr(e1)), Box::new(parse_expr(e2)), Box::new(parse_expr(e3))),
        [Sexp::Atom(S(block)), body @ ..] if block == "block" => {
          if body.is_empty() {
            panic!("Invalid");
          }
          let mut vec = Vec::new();
          for e in body {
            vec.push(parse_expr(e));
          }
          Expr::Block(vec)
        },
        [Sexp::Atom(S(loop_)), e] if loop_ == "loop" => Expr::Loop(Box::new(parse_expr(e))),
        [Sexp::Atom(S(break_)), e] if break_ == "break" => Expr::Break(Box::new(parse_expr(e))),
        [Sexp::Atom(S(let_)), Sexp::List(binds), e] if let_ == "let" => {
          if binds.is_empty() {
            panic!("Invalid");
          }
          let mut vec = Vec::new();
          for bind in binds {
            vec.push(parse_bind(bind));
          }
          Expr::Let(vec, Box::new(parse_expr(e)))
        },
        [Sexp::Atom(S(func_name)), es @ ..] => {
          let mut vec = Vec::new();
          for e in es {
            vec.push(parse_expr(e));
          }
          Expr::Funccall(func_name.to_string(), vec)
        }
        _ => panic!("Invalid"),
      }
    },
    _ => panic!("Invalid"),
  }
}

fn parse_defn(s: &Sexp, func_table: &mut HashMap<String, usize>) -> (Vec<String>, Box<Expr>) {
  match s {
    Sexp::List(vec) => {
      match &vec[..] {
        [Sexp::Atom(S(fun)), Sexp::List(names), expr] if fun == "fun" => {
          if names.is_empty() {
            panic!("Invalid");
          }
          let mut v = Vec::<String>::new(); 
          for name in names {
            match name {
              Sexp::Atom(S(n)) => v.push(n.to_string()),
              _ => panic!("Invalid"),
            }
          }
          match &names[0] {
            Sexp::Atom(S(n)) => {
              if n == "let" || n == "add1" || n == "sub1" || n == "true" || n == "false" || n == "set!" || n == "loop" || n == "break" || n == "if" || n == "block" || n == "input" {
                panic!("Invalid function name");
              }
              if func_table.contains_key(n) {
                panic!("Invalid : Define multiple functions with same name");
              }
              func_table.insert(n.to_string(), names.len() - 1)
            },
            _ => panic!("Invalid"),
          };
          (v, Box::new(parse_expr(expr)))
        },
        _ => panic!("Invalid"),
      }
    }
    _ => panic!("Invalid"),
  }
}

fn parse_prog(s: &Sexp, func_table: &mut HashMap<String, usize>) -> Vec<Statement> {
  match s {
    Sexp::List(vec) => {
      match &vec[..] {
        [defns @ .., expr] => {
          let mut v = Vec::<Statement>::new();
          for defn in defns {
            let p_defn = parse_defn(defn, func_table);
            v.push(Statement::Definition(p_defn.0, p_defn.1));
          }
          v.push(Statement::Expression(Box::new(parse_expr(expr))));
          v
        },
        _ => panic!("Invalid"),
      }
    }
    _ => panic!("Invalid"),
  }
}

fn compile_to_instrs(e: &Expr, si: i64, env: &HashMap<String, i64>, v_args: &HashMap<String, usize>, func_table: &HashMap<String, usize>, l: &mut i64, bl: i64, dep: usize) -> Vec<Instr> {
  let mut v = Vec::<Instr>::new();
  match e {
    Expr::Number(n) => {
      if *n < -4611686018427387904 || *n > 4611686018427387903 {
        panic!("Invalid");
      }
      v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm((*n) * 2)));
    },
    Expr::TRUE => v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(3))),
    Expr::FALSE => v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(1))),
    Expr::INPUT => v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Reg(Reg::RDI))),
    Expr::UnOp(op, subexpr) => {
      v.extend(compile_to_instrs(subexpr, si, env, v_args, func_table, l, bl, dep));
      match op {
        Op1::Add1 => {
          v.push(Instr::Test(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jne(Label::TYPEERROR));
          v.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::Imm(2)));
          v.push(Instr::Jo(Label::OVERFLOW));
        },
        Op1::Sub1 => {
          v.push(Instr::Test(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jne(Label::TYPEERROR));
          v.push(Instr::ISub(Val::Reg(Reg::RAX), Val::Imm(2)));
          v.push(Instr::Jo(Label::OVERFLOW));
        },
        Op1::IsNum => {
          v.push(Instr::Test(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jne(Label::LName(format!("label{}", *l)))); // is not num, return false
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(3)));
          v.push(Instr::Jmp(Label::LName(format!("label{}", *l + 1)))); // is num, jmp out
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l))));
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l + 1))));
          *l += 2;
        },
        Op1::IsBool => {
          v.push(Instr::Test(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jne(Label::LName(format!("label{}", *l)))); // is num, return false
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jmp(Label::LName(format!("label{}", *l + 1)))); // is not num, jmp out
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l))));
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(3)));
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l + 1))));
          *l += 2;
        },
      }
    },
    Expr::BinOp(op, subexpr1, subexpr2) => {
      v.extend(compile_to_instrs(subexpr2, si, env, v_args, func_table, l, bl, dep));
      // check if rax is num (exp2)
      match op {
        Op2::Eq => {},
        _ => {
          v.push(Instr::Test(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jne(Label::TYPEERROR));
        },
      }
      v.push(Instr::IMov(Val::RegOffset(Reg::RSP, si * 8), Val::Reg(Reg::RAX)));
      v.extend(compile_to_instrs(subexpr1, si + 1, env, v_args, func_table, l, bl, dep));
      // check if rax is num (exp1)
      match op {
        Op2::Eq => {
          v.push(Instr::IMov(Val::Reg(Reg::RBX), Val::Imm(0)));
          v.push(Instr::Xor(Val::Reg(Reg::RBX), Val::Reg(Reg::RAX)));
          v.push(Instr::Xor(Val::Reg(Reg::RBX), Val::RegOffset(Reg::RSP, si * 8)));
          v.push(Instr::Test(Val::Reg(Reg::RBX), Val::Imm(1)));
          v.push(Instr::Jne(Label::TYPEERROR));
        },
        _ => {
          v.push(Instr::Test(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jne(Label::TYPEERROR));
        },
      }
      match op {
        Op2::Plus => {
          v.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, si * 8)));
          v.push(Instr::Jo(Label::OVERFLOW));
        },
        Op2::Minus => {
          v.push(Instr::ISub(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, si * 8)));
          v.push(Instr::Jo(Label::OVERFLOW));
        },
        Op2::Times => {
          v.push(Instr::Sar(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::IMul(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, si * 8)));
          v.push(Instr::Jo(Label::OVERFLOW));
        },
        Op2::Lt => {
          v.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, si * 8)));
          v.push(Instr::Jl(Label::LName(format!("label{}", *l)))); // greater, return true
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jmp(Label::LName(format!("label{}", *l + 1)))); // is not greater, jmp out
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l))));
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(3)));
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l + 1))));
          *l += 2;
        },
        Op2::Gt => {
          v.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, si * 8)));
          v.push(Instr::Jg(Label::LName(format!("label{}", *l)))); // smaller, return true
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jmp(Label::LName(format!("label{}", *l + 1)))); // is not smaller, jmp out
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l))));
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(3)));
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l + 1))));
          *l += 2;
        },
        Op2::Ge => {
          v.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, si * 8)));
          v.push(Instr::Jge(Label::LName(format!("label{}", *l)))); // smaller equal, return true
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jmp(Label::LName(format!("label{}", *l + 1)))); // is not smaller equal, jmp out
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l))));
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(3)));
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l + 1))));
          *l += 2;
        },
        Op2::Le => {
          v.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, si * 8)));
          v.push(Instr::Jle(Label::LName(format!("label{}", *l)))); // greater equal, return true
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jmp(Label::LName(format!("label{}", *l + 1)))); // is not greater equal, jmp out
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l))));
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(3)));
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l + 1))));
          *l += 2;
        },
        Op2::Eq => {
          v.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, si * 8)));
          v.push(Instr::Je(Label::LName(format!("label{}", *l)))); // equal, return true
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(1)));
          v.push(Instr::Jmp(Label::LName(format!("label{}", *l + 1)))); // is not equal, jmp out
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l))));
          v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(3)));
          v.push(Instr::Nothing(Label::LName(format!("label{}", *l + 1))));
          *l += 2;
        },
      }
    },
    Expr::Let(vec, body) => {
      let mut nenv = env.clone();
      let mut nsi = si;
      for (x, e) in vec{
        if x == "let" || x == "add1" || x == "sub1" || x == "true" || x == "false" || x == "set!" || x == "loop" || x == "break" || x == "if" || x == "block" || x == "input" {
          panic!("keyword");
        }
        if nenv.contains_key(x) && !env.contains_key(x) {
          panic!("Duplicate binding");
        }
        v.extend(compile_to_instrs(e, nsi, &nenv, v_args, func_table, l, bl, dep));
        nenv = nenv.update(x.to_string(), nsi * 8);
        v.push(Instr::IMov(Val::RegOffset(Reg::RSP, nsi * 8), Val::Reg(Reg::RAX)));
        nsi += 1;
      };
      v.extend(compile_to_instrs(body, nsi, &nenv, v_args, func_table, l, bl, dep));
    },
    Expr::Id(s) => {
      if s == "let" || s == "add1" || s == "sub1" || s == "true" || s == "false" || s == "set!" || s == "loop" || s == "break" || s == "if" || s == "block" {
        panic!("keyword");
      }
      if env.contains_key(s) {
        v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, *env.get(s).unwrap())));
      } else if v_args.contains_key(s) {
        v.push(Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, ((*v_args.get(s).unwrap() + dep + 1) * 8) as i64)));
      } else {
        panic!("Unbound variable identifier {}", s);
      }
    },
    Expr::Set(s, e) => {
      if s == "let" || s == "add1" || s == "sub1" || s == "true" || s == "false" || s == "set!" || s == "loop" || s == "break" || s == "if" || s == "block" || s == "input" {
        panic!("keyword");
      }
      if !env.contains_key(s) {
        panic!("Unbound variable identifier {}", s);
      }
      v.extend(compile_to_instrs(e, si, env, v_args, func_table, l, bl, dep));
      v.push(Instr::IMov(Val::RegOffset(Reg::RSP, *env.get(s).unwrap()), Val::Reg(Reg::RAX)));
    },
    Expr::If(e1, e2, e3) => {
      v.extend(compile_to_instrs(e1, si, env, v_args, func_table, l, bl, dep));
      // v.push(Instr::Test(Val::Reg(Reg::RAX), Val::Imm(1)));
      // v.push(Instr::Je(Label::TYPEERROR)); // if not bool, jump to err
      let v2 = compile_to_instrs(e2, si, env, v_args, func_table, l, bl, dep);
      let v3 = compile_to_instrs(e3, si, env, v_args, func_table, l, bl, dep);
      v.push(Instr::Cmp(Val::Reg(Reg::RAX), Val::Imm(1)));
      v.push(Instr::Je(Label::LName(format!("label{}", *l)))); // if false, jmp to else
      v.extend(v2);
      v.push(Instr::Jmp(Label::LName(format!("label{}", *l + 1)))); // jmp to endif
      v.push(Instr::Nothing(Label::LName(format!("label{}", *l))));
      v.extend(v3);
      v.push(Instr::Nothing(Label::LName(format!("label{}", *l + 1))));
      *l += 2;
    },
    Expr::Block(blk) => {
      for b in blk {
          v.extend(compile_to_instrs(b, si, env, v_args, func_table, l, bl, dep)); 
      }
    },
    Expr::Loop(body) => {
      let curr_l = *l;
      *l += 2;
      v.push(Instr::Nothing(Label::LName(format!("label{}", curr_l))));
      v.extend(compile_to_instrs(body, si, env, v_args, func_table, l, curr_l + 1, dep));
      v.push(Instr::Jmp(Label::LName(format!("label{}", curr_l))));
      v.push(Instr::Nothing(Label::LName(format!("label{}", curr_l + 1))));
    },
    Expr::Break(body) => {
      if bl == -1 {
        panic!("break");
      }
      v.extend(compile_to_instrs(body, si, env, v_args, func_table, l, bl, dep));
      v.push(Instr::Jmp(Label::LName(format!("label{}", bl))));
    },
    Expr::Funccall(func_name, args) => {
      if func_name == "print" {
        if args.len() != 1 {
          panic!("Invalid : func arg num incorrect (print)");
        }
        v.extend(compile_to_instrs(&args[0], si, env, v_args, func_table, l, -1, dep));
        v.push(Instr::Push(Val::Reg(Reg::RDI)));
        v.push(Instr::IMov(Val::Reg(Reg::RDI), Val::Reg(Reg::RAX)));
        v.push(Instr::Push(Val::Reg(Reg::RAX)));
        v.push(Instr::Call(Label::LName("snek_print".to_string())));
        v.push(Instr::Pop(Val::Reg(Reg::RAX)));
        v.push(Instr::Pop(Val::Reg(Reg::RDI)));
      } else {
        match func_table.get(func_name) {
          Some(count) => {
            if args.len() != *count {
              panic!("Invalid : func arg num incorrect");
            }
            v.push(Instr::IMov(Val::RegOnset(Reg::RSP, 8), Val::Reg(Reg::RDI)));
            for (idx, arg) in args.iter().rev().enumerate() {
              v.extend(compile_to_instrs(arg, si, env, v_args, func_table, l, -1, dep));
              v.push(Instr::IMov(Val::RegOnset(Reg::RSP, (idx * 8 + 16) as i64), Val::Reg(Reg::RAX)));
            }
            v.push(Instr::ISub(Val::Reg(Reg::RSP), Val::Imm((args.len() * 8 + 8) as i64)));
            v.push(Instr::Call(Label::LName(func_name.to_string())));
            v.push(Instr::IAdd(Val::Reg(Reg::RSP), Val::Imm((args.len() * 8 + 8) as i64)));
            v.push(Instr::IMov(Val::Reg(Reg::RDI), Val::RegOnset(Reg::RSP, 8)));
          },
          None => panic!("Invalid : No such function"),
        }
      }
    },
  }
  v
}

fn instr_to_str(instr: &Instr) -> String {
  match instr {
    Instr::IMov(v1, v2) => format!("\nmov {}, {}", val_to_str(v1), val_to_str(v2)),
    Instr::IAdd(v1, v2) => format!("\nadd {}, {}", val_to_str(v1), val_to_str(v2)),
    Instr::ISub(v1, v2) => format!("\nsub {}, {}", val_to_str(v1), val_to_str(v2)),
    Instr::IMul(v1, v2) => format!("\nimul {}, {}", val_to_str(v1), val_to_str(v2)),
    Instr::Test(v1, v2) => format!("\ntest {}, {}", val_to_str(v1), val_to_str(v2)),
    Instr::Cmp(v1, v2) => format!("\ncmp {}, {}", val_to_str(v1), val_to_str(v2)),
    Instr::Sar(v1, v2) => format!("\nsar {}, {}", val_to_str(v1), val_to_str(v2)),
    Instr::Xor(v1, v2) => format!("\nxor {}, {}", val_to_str(v1), val_to_str(v2)),
    Instr::Push(v1) => format!("\npush {}", val_to_str(v1)),
    Instr::Pop(v1) => format!("\npop {}", val_to_str(v1)),
    Instr::Jmp(l1) => format!("\njmp {}", label_to_str(l1)),
    Instr::Jne(l1) => format!("\njne {}", label_to_str(l1)),
    Instr::Je(l1) => format!("\nje {}", label_to_str(l1)),
    Instr::Jg(l1) => format!("\njg {}", label_to_str(l1)),
    Instr::Jl(l1) => format!("\njl {}", label_to_str(l1)),
    Instr::Jge(l1) => format!("\njge {}", label_to_str(l1)),
    Instr::Jle(l1) => format!("\njle {}", label_to_str(l1)),
    Instr::Jo(l1) => format!("\njo {}", label_to_str(l1)),
    Instr::Nothing(l1) => format!("\n{}:", label_to_str(l1)),
    Instr::Call(l1) => format!("\ncall {}", label_to_str(l1)),
  }
}

fn val_to_str(val: &Val) -> String {
  match val {
    Val::Imm(num) => format!("{}", *num),
    Val::Reg(Reg::RAX) => format!("rax"),
    Val::Reg(Reg::RBX) => format!("rbx"),
    Val::Reg(Reg::RSP) => format!("rsp"),
    Val::Reg(Reg::RDI) => format!("rdi"),
    Val::RegOffset(Reg::RSP, offset) => format!("[rsp + {}]", offset),
    Val::RegOnset(Reg::RSP, onset) => format!("[rsp - {}]", onset),
    _ => panic!("cannot convert val to str"),
  }
}

fn label_to_str(label: &Label) -> String {
  match label {
    Label::TYPEERROR => format!("TYPEERROR"),
    Label::OVERFLOW => format!("OVERFLOW"),
    Label::LName(st) => st.to_string(),
  }
}

fn compile(e: &Expr, v_args: &HashMap<String, usize>, func_table: &HashMap<String, usize>, label: &mut i64, dep: usize) -> String {
  let mut s = String::new();
  let v = compile_to_instrs(e, 0, &HashMap::new(), v_args, func_table, label, -1, dep);
  for i in v {
    s.push_str(&instr_to_str(&i));
  }
  s
}

fn depth(e: &Expr) -> usize {
  match e {
    Expr::Number(_) | Expr::TRUE | Expr::FALSE | Expr::INPUT | Expr::Id(_)=> 0,
    Expr::Let(vec, e1) => {
      let mut ma = 0;
      for (idx, (x, e)) in vec.iter().enumerate() {
        ma = ma.max(depth(e) + idx);
      }
      ma = ma.max(depth(e1) + vec.len());
      ma
    },
    Expr::UnOp(op, e1) => depth(e1),
    Expr::BinOp(op, e1, e2) => (depth(e1) + 1).max(depth(e2)),
    Expr::Set(id, e1) => depth(e1),
    Expr::If(e1, e2, e3) => depth(e1).max(depth(e2).max(depth(e3))),
    Expr::Block(vec) => {
      let mut ma = 0;
      for e in vec {
        ma = ma.max(depth(e));
      }
      ma
    },
    Expr::Loop(e1) => depth(e1),
    Expr::Break(e1) => depth(e1),
    Expr::Funccall(id, vec) => {
      let mut ma = 0;
      for e in vec {
        ma = ma.max(depth(e));
      }
      ma
    },
  }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let in_name = &args[1];
    let out_name = &args[2];

    // You will make result hold the result of actually compiling
    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    in_contents = format!("({})", in_contents);
    let s_expr = match parse(&in_contents) {
      Ok(expr) => expr,
      Err(_) => panic!("Invalid"),
    };
    
    let mut func_table = HashMap::new();

    let v_prog = parse_prog(&s_expr, &mut func_table);
    // let expr = parse_expr(&s_expr);

    let mut result = String::new();
    let mut label = 0;
    if let Some((expr, defns)) = v_prog.split_last() {
      for defn in defns {
        match &defn {
          Statement::Definition(names, expr) => {
            if let Some((func_name, args)) = names.split_first() {
              result.push_str(&format!("\n{}:", func_name));
              let dep = depth(expr) + 2;
              result.push_str(&format!("\n  sub rsp, {}", dep * 8));
              let mut v_args = HashMap::new();
              for (idx, arg) in args.iter().enumerate() {
                if arg == "let" || arg == "add1" || arg == "sub1" || arg == "true" || arg == "false" || arg == "set!" || arg == "loop" || arg == "break" || arg == "if" || arg == "block" || arg == "input" {
                  panic!("Invalid arg name");
                }
                if v_args.contains_key(arg) {
                  panic!("Duplicate arg name");
                }
                v_args.insert(arg.to_string(), idx);
              }
              result.push_str(&compile(expr, &v_args, &func_table, &mut label, dep));
              result.push_str(&format!("\n  add rsp, {}", dep * 8));
              result.push_str(&format!("\n  ret"));
            }
          },
          _ => panic!("Invalid"), // other than last one is not defn
        }
      }
      match &expr {
        Statement::Expression(e) => {
          let dep = depth(e) + 2;
          result.push_str(&format!("\nour_code_starts_here:"));
          result.push_str(&format!("\nsub rsp, {}", dep * 8));
          result.push_str(&compile(e, &HashMap::new(), &func_table, &mut label, dep));
          result.push_str(&format!("\nadd rsp, {}", dep * 8));
          result.push_str(&format!("\n  ret"));
        },
        _ => panic!("Invalid"), // last one is not expr
      }
    }

    let asm_program = format!(
        "
section .text
extern snek_error
extern snek_print
global our_code_starts_here
  {}
TYPEERROR:
  mov rdi, 1
  push rsp
  call snek_error
OVERFLOW:
  mov rdi, 2
  push rsp
  call snek_error
",
        result
    );

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}
