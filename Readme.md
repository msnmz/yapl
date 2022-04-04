# yapl (yet another programming language)

A programming language project to get to know lalrpop.

For now this is just an interpreter, will contain an virtual machine, hopefully.

All the supported things are listed in ./lang/test.txt

## Supported features until now

- built-in `print` function to debug features
- basic operators:
  - assignment (=)
  - logical operators (<, >, ==, !=, >=, <=)
  - arithmetic operators -with c-like precedence rules- (+, -, /, \*, %)
  - Arithmetic assignments (+=,-=,\*=,/=,%=)
- String literals
- Integers (i64)
- comments (#)
- Rest, Spread operators (..identifier)
- basic pattern matching with discard support (with underscore \_)
- if & if else (for now if else if does not work)
- loops with `while`

## Local development

```sh
find src lang | grep -v '^src/\.' | entr -sc 'cargo build && target/debug/yapl'
```

looks for the changes both in lang/ and src/
