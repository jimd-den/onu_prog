# Ọ̀nụ

Ọ̀nụ (meaning "voice" or "utterance") is a Turing-complete programming language whose syntax is designed as a **Formal Discourse**. It enforces strict Subject-Verb-Object (SVO) topology and the **Agency Principle**, where code is expressed as a series of active, direct, and verifiable events.

## Usage

To execute an Ọ̀nụ discourse unit, use the standard command:

```bash
cargo run -- <filename>.onu
```

## The Agency Principle

In Ọ̀nụ, code is not a set of passive instructions. Every behavior **takes** what it needs and **delivers** what it must. Variables are not "set"; they are **derived** from expressions.

### Discourse Structure

Code is organized into modules focusing on a single concern.

```onu
the module called GreetingDiscourse
    with concern: introductory broadcast

the effect behavior called run
    with intent: program entry point
    takes: nothing
    delivers: nothing
    as:
        broadcasts "Hello, World!"
```

### Derivations (State)

Values are immutable and established through explicit derivation.

```onu
derivation: x derives-from an integer 10
derivation: y derives-from a float 3.14
derivation: message derives-from a string "Hello"
```

### Behaviors (Active Logic)

Behaviors are defined by their intent and the transactional relationship with their provisions.

```onu
the behavior called scale-value
    with intent: transform a number by a factor
    takes:
        an integer called input
        an integer called factor
    delivers: an integer
    as:
        input scales-by factor
```

### Conditionals

Decision making follows the logical flow of a proposition.

```onu
if x exceeds 5
    then broadcasts "Threshold exceeded"
    else broadcasts "Within limits"
```

### Recursive Growth (Loops)

Repetition is achieved through recursive behaviors with proven termination.

```onu
the effect behavior called countdown
    with intent: count down to equilibrium
    takes:
        an integer called n
    delivers: nothing
    with diminishing: n
    as:
        if n matches 0
            then broadcasts "Equilibrium reached."
            else
                derivation: dummy derives-from nothing broadcasts (n utilizes as-text)
                derivation: next  derives-from an integer n decreased-by 1
                next utilizes countdown
```

### Active Operations

All interactions utilize semantic verbs rather than abstract symbols.

*   **Arithmetic:** `added-to`, `decreased-by`, `scales-by`, `partitions-by`
*   **Logic:** `unites-with`, `joins-with`, `opposes`
*   **Comparisons:** `matches`, `exceeds`, `falls-short-of`
*   **I/O:** `broadcasts`
*   **Agency:** `utilizes`, `acts-as`, `derives-from`
