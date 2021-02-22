# a Computer Algebra System project in rust

Still work in progress.
Just a simple calculator for now.

### cas-cli example usage
`cas-cli EXPR1 EXPR2 EXPR3 ...`
```
$ cargo run --example=cas-cli '(1-5)*2+3(2(2+2))/2*2+2' '1/2+2/4'
> Parsed: + -> [ * -> [ - -> [ 1, 5 ], 2 ], + -> [ * -> [ / -> [ * -> [ 3, * -> [ 2, + -> [ 2, 2 ] ] ], 2 ], 2 ], 2 ] ]
  Evaluated: 18
  Parsed: + -> [ / -> [ 1, 2 ], / -> [ 2, 4 ] ]
  Evaluated: 1
```
```
$ cargo run --example=cas-cli -- -vv '5*4+3'
> Parsed: + -> [ * -> [ 5, 4 ], 3 ]
  Evaluated: 23
```

TODO:
- factorizer
- solver
- simplifier
- rationals (to fix floating point errors with for ex. '1/3+1/3+1/3-1' being equal to '-0.00000000000000011102230246251565')



https://stackoverflow.com/questions/7540227/strategies-for-simplifying-math-expressions

https://en.wikipedia.org/wiki/Factorization_of_polynomials

https://en.wikipedia.org/wiki/Shunting-yard_algorithm

https://en.wikipedia.org/wiki/Quadratic_formula

https://en.wikipedia.org/wiki/Cubic_equation
