# Ọ̀nụ

Ọ̀nụ is a programming language.

## Usage

To run a script, use the cargo command:

```bash
cargo run -- <filename>.onu
```

## Syntax

### Basics

Code is organized into behaviors within a module.

```
the module called Example
    with concern: demonstration

the effect behavior called main
    with intent: program entry point
    receiving:
    returning: nothing
    as:
        emit "Hello, World!"
```

### Variables

Variables are immutable and declared with `let`.

```
let x is an integer 10
let y is a float 3.14
let message is a strings "Hello"
```

### Behaviors (Functions)

Behaviors are defined with inputs and outputs.

```
the behavior called add-numbers
    with intent: sum two integers
    receiving:
        an integer called a
        an integer called b
    returning: an integer
    as:
        a added-to b
```

### Conditionals

Use `if`, `then`, and `else`.

```
if x is-greater-than 5
    then emit "Greater"
    else emit "Smaller"
```

### Loops (Recursion)

Loops are achieved through recursion.

```
the behavior called countdown
    with intent: count down from n
    receiving:
        an integer called n
    returning: nothing
    with diminishing: n
    as:
        if n is-zero
            then emit "Done"
            else
                emit (n as-text)
                let next is an integer n decreased-by 1
                next countdown
```

### Standard Operations

Operations are written as words.

*   **Math:** `added-to`, `subtracted-from`, `multiplied-by`, `divided-by`
*   **Logic:** `both-true`, `either-true`, `not-true`
*   **Comparisons:** `is-equal-to`, `is-greater-than`, `is-less-than`, `is-zero`
