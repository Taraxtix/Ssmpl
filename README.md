# Ssmpl

A basic stack based programming language (For learning purposes).

## Goals
Those goals will surely change during the project. 
- [X] Simulated
- [X] Compiled
- [X] Statically typed (with compilation time type checking)
- [ ] Turing complete
- [ ] Self-Hosted

## Documentation

### Random Informations
- Parentheses are ignored

### --Comments--
Comments will use `#` as a delimiter.
```
# This is a line comment
Code Here # This is an inline comment # Code here
```

Note that for an inline comment to start, a space is mandatory after the last operation.
For now it is impossible to use `#` inside of a comment (I think about making it possible by escaping it)

### --Push--
To push a digit to the stack you just have to write the digit as is.
```pascal
42 69
```

### --Dump--
`dump` 
```rust
let a = pop();
println!("{a}");
```

### --Arithmetic--
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
`drop`
```rust
pop();
```
#### --2Drop--
`2drop`
```rust
pop();
pop();
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
`over`
```rust
let a = pop();
let b = pop();
push(b);
push(a);
push(b);
```
#### --2Over--
`2over`
```rust
let a = pop();
let b = pop();
let c = pop();
push(c);
push(b);
push(a);
push(c);
```
#### --Dup--
`dup`
```rust
let a = pop();
push(a);
push(a);
```
### --Control flow--
In all control flow, the `condition` part must add exactly one value to the stack without modifying the rest of the stack.
This value will be consider `false` if equal to zero and `true` otherwise
#### --If--
```rust
if # condition # do
    # Execute here if condition is true #
end
```

#### --While--
Using a while, the inside block of code must return a stack containing the same amout of value as before the while. 
```rust
while # condition # do
    # Execute here while condition is true #
end
```

#### --Else--
using a else block, the if block and the else block must alter the stack the same way. 
```rust
if # condition # do
    # Execute here if condition is true #
else
    # Execute here if condition is false #
end
```
### --Comparison--
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
