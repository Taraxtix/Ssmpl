# Ssmpl

A basic stack based programming language (For learning purposes).

## Goals

Those goals will surely change during the project.

- [x] Simulated
- [x] Compiled
- [x] Statically typed (with compilation time type checking)
- [x] Turing complete (See examples/rule110.ssmpl [[Rule 110](https://en.wikipedia.org/wiki/Rule_110)])
- [ ] Self-Hosted

## Quick Start

First clone the repository, Then you'll can compile the compiler using one of the following commands.

`make release` while compile the compiler as release and add a link to the current directory named `ssmpl`
`make` or `make debug` will provide a debug version of the compiler (Also add a link to the current directory)

## Documentation

### Implicit casting

Some types can implicitly be casted to other types at compiled time.

- `bool` can be casted from `i64` and `f64`
- `i64` can be casted from `bool` and `ptr`
- `f64` can be casted from `i64` and `bool`
- `ptr` can be casted from `i64`

### Explicit casting

`cast(TYPE)` cast the top element of the stack to the given type.

With explicit casting you can cast any type to any other type.
Casting to `bool` alter the bits of the value while any other cast only affect the behavior of future operations.

### --Size Arguments--

Some operations can take a size argument marked as `SIZE` in the documentation.
Those size arguments can be either a positive integer literal or a macro consisting of a single positive integer literal.

### --Comments--

Comments will use `//` for line comments and `/*` and `*/` for block comments as a delimiter.

```rust
// This is a comment
Code Here /* This is an inline comment */ Code here
/* This is a
multiline comment */
```

### --Push--

To push a digit to the stack you just have to write the digit as is.
You can pass integers as well as floats.
values prefixes with `0b`, `0o`, `0x` are supported and will be converted to integers from binary, octal and hex respectively.:

```rust
42 69. 420.12 -9.9 .690 -10 0xBEEF 0b101 0o707
```

### --Dump--

``dump`
pop the top element of the stack and print it followed by a new line.

### --Arithmetic--

Any operation between a integer and a float will implicitly convert the integer to a float.

#### --Plus--

`+`

```rust
let b = pop();
let a = pop();
push(a + b);
```

#### --Minus--

`-`

```rust
let b = pop();
let a = pop();
push(a - b);
```

#### --Multiply--

`*`

```rust
let b = pop();
let a = pop();
push(a * b);
```

#### --Division--

Division is a bit special as it push both the result and the reminder of the division to the stack.
`/`

```rust
let b = pop();
let a = pop();
push(a / b);
```

#### --Modulo--

Modulo is only supported for integers.
`%`

```rust
let b = pop();
let a = pop();
push(a % b);
```

#### --Increment--

`++`

```rust
let a = pop();
push(a + 1);
```

#### --Decrement--

`--`

```rust
let a = pop();
push(a - 1);
```

### --Stack Manipulation--

#### --Drop--

`drop(SIZE)` (pops `SIZE` times from the stack)
If no `SIZE` is provide (`drop`), it is equivalent to `drop1`

```rust
for _ in 0..SIZE{
    pop();
}
```

#### --Swap--

`swap`

```rust
let a = pop();
let b = pop();
push(a);
push(b);
```

#### --Over--

`over(SIZE)` Where `SIZE` is a positive integer (push a copy of the `SIZE+1`nth element of the stack)
If no `SIZE` is provide (`over`), it is equivalent to `over1`

```rust
let a = stack[SIZE+1];
push(a);
```

#### --SetOver--

`setOver(SIZE)` Where `SIZE` is a positive integer (set the `SIZE+1`nth element of the stack to the top of the stack)

#### --Dup--

`dup(SIZE)` Where `SIZE` is a positive integer (push a copy of the `SIZE` firsts elements of the stack (in the same order))

```rust
for _ in 0..SIZE{
    push(stack[SIZE]);
}
```

### --Control flow--

In all control flow, the `condition` part must add exactly one boolean value to the stack without modifying the rest of the stack.

#### --If--

An If block must return the stack as it was before the if block.

```rust
if # condition # do
    # Execute here if condition is true #
end
```

#### --While--

A while block must return as it was before the while block.

```rust
while # condition # do
    # Execute here while condition is true #
end
```

#### --Else--

An if with an else block must modify the stack in the same way

```rust
if # condition # do
    # Execute here if condition is true #
else
    # Execute here if condition is false #
end
```

### --Comparison--

All comparison operators push a boolean value to the stack.

WARNING: Comparing floats uses the CPU floats arithmetic so it can be off cause to precision error.

#### --Equal--

`==`

```rust
let b = pop();
let a = pop();
push(a == b);
```

#### --Not Equal--

`!=`

```rust
let b = pop();
let a = pop();
push(a != b);
```

#### --Greater--

`>`

```rust
let b = pop();
let a = pop();
push(a > b);
```

#### --Greater or Equal--

`>=`

```rust
let b = pop();
let a = pop();
push(a >= b);
```

#### --Less--

`<`

```rust
let b = pop();
let a = pop();
push(a < b);
```

#### --Less or Equal--

`<=`

```rust
let b = pop();
let a = pop();
push(a <= b);
```

### --Program arguments--

#### --Argc--

`argc`: Pushes the number of arguments passed to the program

#### --Argv--

`argv`: Pushes the pointer to the start of the arguments passed to the program

### --Memory manipulation--

#### --Load--

`<|X` Where `X` is the size of the value to load from memory. (Possible values: 8, 16, 32, 64)

```rust
let ptr = pop();
push(memory[ptr]);
```

#### --Store--

`|>X` Where `X` is the size of the value to store to the memory. (Possible values: 8, 16, 32, 64)

```rust
let value = pop();
let ptr = pop();
memory[ptr] = value;
```

### --Macro--

Macro are replaced by their value at compile time.

```rust
macro NAME {
    OPERATIONS
}
```

### --Include--

`include "file_path"`
file path is relative to where the compiler is executed. (I will try to make it relative to the file in the future)

Parse the file and append everything it contains to main program

### --Logical Operation--

#### --Logical And--

`&&` If the first two element of the stack evaluates to `true` then `true` is pushed to the stack, otherwise `false` is pushed.

```rust
let b = pop();
let a = pop();
push(a && b);
```

#### --Logical Or--a

`||` If one of the first two element of the stack evaluates to `true` then `true` is pushed to the stack, otherwise `false` is pushed.

```rust
let b = pop();
let a = pop();
push(a || b);
```

### --Bitwise Operation--

#### --Bitwise And--

`&` Pushes the bitwise AND of the top two elements of the stack to the stack.

#### --Bitwise Or--

`|` Pushes the bitwise OR of the top two elements of the stack to the stack.

#### --Shift Left--

`<<` Pushes the second element of the stack shifted left by the top element of the stack.

```rust
let b = pop();
let a = pop();
push(a << b);
```

#### --Shift Right--

`>>` Pushes the second element of the stack shifted right by the top element of the stack.

```rust
let b = pop();
let a = pop();
push(a >> b);
```

### --Memory Access--

#### --Free Memory--

Ssmpl has 1024 bytes of free memory which you can do whatever you want with.

`mem` pushes the pointer to the start of the free memory.

#### --Named Memory--

`decla NAME SIZE` declares a memory region with named NAME of size SIZE.
`mem(NAME)` pushes the pointer to the start of the named memory region.
