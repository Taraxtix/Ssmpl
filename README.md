# Ssmpl

A basic stack based programming language (For learning purposes).

## Goals
Those goals will surely change during the project. 
- [X] Simulated
- [X] Compiled
- [ ] Turing complete
- [ ] Statically typed (with compilation time type checking)
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
```
42 69
```

### --Dump--
`dump` 
```
let a = pop();
println!(a);
```

### --Arithmetic--
#### --Plus--
`+`
```
let b = pop();
let a = pop();
push(a + b);
```
#### --Minus--
`-`
```
let b = pop();
let a = pop();
push(a - b);
```
#### --Multiply--
`*`
```
let b = pop();
let a = pop();
push(a * b);
```
#### --Division--
Division is a bit special as it push both the result and the reminder of the division to the stack.
`/`
```
let b = pop();
let a = pop();
push(a / b);
push(a % b);
```
#### --Increment--
`++`
```
let a = pop();
push(a + 1);
```
#### --Decrement--
`--`
```
let a = pop();
push(a - 1);
```
### --Stack Manipulation--
#### --Drop--
`drop`
```
pop();
```
#### --2Drop--
`2drop`
```
pop();
pop();
```
#### --Swap--
`swap`
```
let a = pop();
let b = pop();
push(a);
push(b);
```
#### --Over--
`over`
```
let a = pop();
let b = pop();
push(b);
push(a);
push(b);
```
#### --2Over--
`2over`
```
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
```
let a = pop();
push(a);
push(a);
```
