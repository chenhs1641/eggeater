# The Egg Eater Language

## Concrete Syntax

The concrete syntax of Egg Eater has added `tuple` and `index` expressions from the past Diamondback.

```
<prog> := <defn>* <expr>
<defn> := (fun (<name> <name>*) <expr>)
<expr> :=
  | <number>
  | true
  | false
  | input
  | <identifier>
  | (let (<binding>+) <expr>)
  | (<op1> <expr>)
  | (<op2> <expr> <expr>)
  | (set! <name> <expr>)
  | (if <expr> <expr> <expr>)
  | (block <expr>+)
  | (loop <expr>)
  | (break <expr>)
  | (<name> <expr>*)
  | (tuple <expr>+) (new)
  | (index <expr> <expr>) (new)


<op1> := add1 | sub1 | isnum | isbool | print
<op2> := + | - | * | < | > | >= | <= | =

<binding> := (<identifier> <expr>)
```

## Semantics

The semantics of added features is as follows:

### 1. Tuples:

Tuples are created using the `(tuple <expr>+)` syntax.
They can hold any number of expressions and dynamically allocate memory to store the evaluated values. Tuples allow heap-allocation of an arbitrary number of values. Expressions within the tuple are evaluated, and memory is allocated accordingly.

### 2. Indexed Lookup:

Indexed lookup is performed using the `(index <expr> <expr>)` syntax.
The first expression represents a heap-allocated value, and the second expression is the index.
The lookup retrieves the value at the specified index.
If the index is out of bounds, a dynamic error is reported.