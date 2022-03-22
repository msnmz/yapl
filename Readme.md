# yapl

This is not serious stuff. Just to get familiar with lalrpop.

For now just interpreter, will contain an virtual machine, hopefully.

All the supported things are listed in ./lang/test.txt

To run locally:

```sh
find src lang | grep -v '^src/\.' | entr -sc 'cargo build && target/debug/yapl'
```

looks for the changes both in lang/ and src/
