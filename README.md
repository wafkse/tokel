# Tokel

**Tokel** is a procedural macro library (provided as a *standalone component* for use in other procedural macro libraries and as its own *derive* crate) that exposes a centralized method to manipulate a token-stream.

**Tokel** employs a recursive, *expansion block*-based model with *transformer* pipelines attached to the end of each *expansion block*.

## The Expansion Model

*Tokel* takes a singular top-level token-stream (a "TokelStream" in the formal grammar definition) and performs recursive  descent to find and evaluate these blocks.

Expansion blocks are denoted by `[< ... >]`. Any tokens outside of an expansion block are ignored and emitted as-is.

When Tokel encounters an expansion block, it evaluates it **bottom-up (inside-out)**. Any nested blocks are fully expanded before the outer block is processed. Once the inner token-stream is resolved, it is passed through the transformer pipeline attached to the block.

### Transformers

Transformers are pure functions that map an input token-stream to an output token-stream. They are chained to the end of an expansion block using a colon (`:`).

If a transformer takes arguments, they are provided using double brackets `[[ ... ]]`. The arguments themselves are parsed as a standard Tokel token-stream, meaning you can nest expansion blocks inside them.

```ignore
// Expands to `GetMyIdentifier`
[< my_identifier >]:case[[pascal]]:prefix[[Get]]
```

Because transformers only apply to the resolved output of a `[< ... >]` block, the syntax avoids the ambiguity of attaching transforms to individual tokens.

### Formal Grammar

```ebnf,ignore
TokelStream       ::= Element*

Element           ::= "[" Block "]" Pipeline
                    | TokelTree

(* NOTE: A `None`-delimited group is left unchanged. *)
TokelTree         ::= "(" TokelStream ")"
                    | "{" TokelStream "}"
                    | "[" TokelStream "]"
                    | Ident | Literal | Punct

Block             ::= "<" TokelStream ">"

Pipe              ::= ":" Transformer

Pipeline          ::= Pipe*

Transformer       ::= Ident ( "[[" TokelStream "]]" )?
```

# License

Copyright (C) 2026 W. Frakchi

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

See [the full license agreement](./LICENSE) for further information.
