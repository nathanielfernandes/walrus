# walrus

Called this walrus because of the `:=` operator, which is a walrus.

WAM (Warren Abstract Machine) experiments in Rust, following the wam-book.

Eventually I want to be able to compile and run a subset of Prolog programs within rust.

## L0

```
query: p(Z, h(Z, W), f(W))
```

correctly compiles to

```
put_structure h/2, X3
set_variable X2
set_variable X5
put_structure f/1, X4
set_value X5
put_structure p/3, X1
set_value X2
set_value X3
set_value X4
```
