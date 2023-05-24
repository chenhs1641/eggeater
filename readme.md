# The Egg Eater Language

## 1. Concrete Syntax

The concrete syntax of Egg Eater has added `nil` value, `tuple` and `index` expressions from the past Diamondback.

```
<prog> := <defn>* <expr>
<defn> := (fun (<name> <name>*) <expr>)
<expr> :=
  | <number>
  | nil                            (new)
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
  | (tuple <expr>+)                (new)
  | (index <expr> <expr>)          (new)


<op1> := add1 | sub1 | isnum | isbool | print
<op2> := + | - | * | < | > | >= | <= | =

<binding> := (<identifier> <expr>)
```

## 2. Semantics

The semantics of added features is as follows:

### 2.1. Tuples

Tuples are created using the `(tuple <expr>+)` syntax.

They can hold any number of expressions and dynamically allocate memory to store the evaluated values. Tuples allow heap-allocation of an arbitrary number of values. Expressions within the tuple are evaluated, and memory is allocated accordingly.

If a heap-allocated value is the result of a program or printed by print, all of its contents will be printed to show its structure.

### 2.2. Indexed Lookup

Indexed lookup is performed using the `(index <expr> <expr>)` syntax.

The first expression represents a heap-allocated value, and the second expression is the index.

The lookup retrieves the value at the specified index.

If the index is out of bounds, a dynamic error is reported.

### 2.3. Nil

`nil` is a value that represents the absence of a meaningful or valid object or data. It has the same type-tag as tuples.

### 2.4. Errors

Beyond Diamondback, several dynamic error types are added in Egg Eater.

- If the operators other than `=` are used on any heap-allocated values, an error containing "invalid argument" will be raised from the running program.
- If an out-of-bounds index is given in the `index` expression, an error containing "index out of bound" and the given index causing error will be raised.
- If the program tries to index into a `nil` object, an error containing "try to index of nil" will be raised.

## 3. Heap-allocated Values Arrangement

The heap-allocated values are arranged as follows:

There first comes an 8-byte value representing the size of the tuple, followed by some (the number is the same as the size of the tuple) 8-byte values representing the element of the tuple.

For example, suppose the heap allocated address starts at `0x1000`, and we have `(tuple 1 2 3)`, the diagram of the values will look like:

| 0x1000   | 0x1008 | 0x1010 | 0x1018 |
| ---      | ---    | ---    | ---    |
| 3 (size) | 2 (1)  | 4 (2)  | 6 (3)  |

For nesting tuples, the inner tuples will be allocated memory first.

For example,suppose the heap allocated address starts at `0x1000`, and we have `(tuple 1 2 (tuple 3 4))`, the diagram of the values will look like:

| 0x1000   | 0x1008 | 0x1010 | 0x1018   | 0x1020 | 0x1028 | 0x1030 |
| ---      | ---    | ---    | ---      | ---    | ---    | ---    |
| 2 (size) | 6 (3)  | 8 (4)  | 3 (size) | 2(1)    | 4 (2)  | 0x1001 |

## 4. Required Tests

### 4.1. `simple_example.snek`

This is a simple example that prints two tulples (it will print the second twice since it is the result of the program).
```
(let ((a (tuple 1 2 3)) (b (tuple 4 5 6))) (block (print a) (print b)))
```

```
> ./tests/simple_examples.run 
(tuple 1 2 3)
(tuple 4 5 6)
(tuple 4 5 6)
```

### 4.2. `error_tag.snek`

This is an example that tries to compare between two tuples and gets an error.

```
(< (tuple 2 3) (tuple 2))
```

```
> ./tests/error_tag.run 
invalid argument
```

The runtime catches the error when evaluating `<` operation. It checks the type of the oprands. After finding the program tries to compare between two tuples, it jumps into the error handling code.

### 4.3. `error_bound.snek`

This is an example that tries to index out of bound and gets an error.

```
(index (tuple 1 2 3) 4)
```

```
> ./tests/error_bounds.run     
index out of bound, 4
```

The runtime catches the error when evaluating `index`. It compares the size of the tuple and the intended index. When discovering index out of bound, it jumps into the error handling code.

### 4.4. `error3.snek`

This is an example that tries to index into a `nil` object and gets an error.

```
(index nil 2)
```

```
> ./tests/error3.run 
try to index of nil
```

The runtime catches the error when evaluating `index`. It checks whether the heap-allocated object is `nil`, if so, it jumps into the error handling code.

### 4.5. `points.snek`

This is a program with a function (`point`) that takes an `x` and a `y` coordinate and produces a structure with those values, and a function (`add2`) that takes two points and returns a new point with their `x` and `y` coordinates added together, along with several tests that print example output from calling these functions.

```
(fun (point x y) (tuple x y))
(fun (takex pnt) (index pnt 1))
(fun (takey pnt) (index pnt 2))
(fun (add2 pnt1 pnt2) (point (+ (takex pnt1) (takex pnt2)) (+ (takey pnt1) (takey pnt2))))
(let (
  (a (point 2 3)) (b (point 4 5)) (c (point 6 7))
) (
    block (
      print (add2 a b)
    ) (
      print (add2 b c)
    ) (
      add2 a c
    )
))
```

```
> ./tests/points.run 
(tuple 6 8)
(tuple 10 12)
(tuple 8 10)
```

### 4.6. `bst.snek`

This is a program that builds a binary search trees, and implements functions to add an element and check if an element is in the tree. The program includes several tests that print example output from calling these functions.

```
(fun (value bst) (index bst 1))
(fun (left bst) (index bst 2))
(fun (right bst) (index bst 3))
(fun (node le ri el) (tuple le ri el))
(fun (find bst elt) (
    if (= bst nil) false (
        if (> elt (value bst)) (
            find (right bst) elt
        ) (
            if (< elt (value bst)) (
                find (left bst) elt
            ) true
        )
    )
))
(fun (insert bst elt) (
    if (= bst nil) (
        node elt nil nil
    ) (
        if (> elt (value bst)) (
            node (value bst) (left bst) (insert (right bst) elt)
        ) (
            if (< elt (value bst)) (
                node (value bst) (insert (left bst) elt) (right bst)
            ) bst
        )
    )
))
(let ((bst (tuple 4 (tuple 2 (tuple 1 nil nil) (tuple 3 nil nil)) (tuple 6 (tuple 5 nil nil) (tuple 7 nil nil))))) (
    block (
        print (insert bst 0)
    ) (
        print (insert bst 8)
    ) (
        print (find bst 5)
    ) (
        find bst 20
    )
))
```

```
> ./tests/bst.run
(tuple 4 (tuple 2 (tuple 1 (tuple 0 nil nil) nil) (tuple 3 nil nil)) (tuple 6 (tuple 5 nil nil) (tuple 7 nil nil)))
(tuple 4 (tuple 2 (tuple 1 nil nil) (tuple 3 nil nil)) (tuple 6 (tuple 5 nil nil) (tuple 7 nil (tuple 8 nil nil))))
true
false
```

## 5. Comparison with TWO Other Programming Languages

### 5.1. Tuples in Python
In Python, tuples are immutable sequences, meaning their elements cannot be modified once created. When a tuple object is created, the Python interpreter calculates the required memory size to accommodate the tuple and allocates a contiguous block of memory on the heap. The individual elements of the tuple are allocated within the same block of memory, preserving their order. The values of the elements are assigned within the allocated memory block. After creation, a reference to the allocated memory block is returned.

### 5.2. Vectors in C++
In C++, vectors are dynamic arrays that can grow or shrink in size at runtime. When a vector is declared, the C++ compiler generates code that automatically allocates memory for the vector on the heap. Initially, the total number of elements the vector can hold without resizing (capacity) is determined based on the specified or default initial size. The vector object contains a pointer to the dynamically allocated memory block, which is initially empty. C++ allows adding new elements to a vector or removing existed elements from one, with it resized on demand efficiently.

### 5.3. Comparison with Tuples in the Egg Eater Language

Since the tuples cannot be changed after creation, they are more like tuples in Python than vectors in C++. Also, the heap memory allocated to tuples will not change after their allocation, which also makes them more like tuples in Python.

## 6. References

[How to Insert into a BST without Modifying Existed Data Structure](https://edstem.org/us/courses/38748/discussion/3125816)

[Memory Allocation of Vectors in C++](https://stackoverflow.com/questions/10366474/where-does-a-stdvector-allocate-its-memory)

[Memory Allocation of Tuples in Python](https://www.opensourceforu.com/2021/05/memory-management-in-lists-and-tuples/)