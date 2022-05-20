# blox

[![pipeline status](https://git.gvk.idi.ntnu.no/course/prog2006/as/benjabj/individual-project/badges/master/pipeline.svg)](https://git.gvk.idi.ntnu.no/course/prog2006/as/benjabj/individual-project/-/commits/master)
[![coverage report](https://git.gvk.idi.ntnu.no/course/prog2006/as/benjabj/individual-project/badges/master/coverage.svg)](https://git.gvk.idi.ntnu.no/course/prog2006/as/benjabj/individual-project/-/commits/master)

This is a rust implementation of the lox language by Rober Nystrom, introduced in his book [Crafting Interpreters](https://craftinginterpreters.com/)

It only covers the book up to the chapter on **Calls and Functions** due to the lack of time and prioritization on tests and documentation.

The implementation features a **bytecode compiler** and a stack-based **interpreter**.

## Dependencies
- rust
- cargo
- tarpaulin (if you want to run with test coverage)

You can install rust and cargo via [rustup](https://rustup.rs/)

Tarpaulin can be found [here](https://github.com/xd009642/tarpaulin)


**Note**: Tarpaulin does not work on windows.



## Building
```
cargo build
```
Release mode is recommended if you want to run benchmarks or performance heavy tasks.
```
cargo build --release
```

Now you should be able to run the program from the ./target/{build-config} directory:
```
./blox
```
Alternatively run it with cargo:
```
cargo run --release
```

This will start the REPL and you can start running code!
Try running the following command:
```
print "Hello, World!";
```
It should print "Hello, World!" to the console.

If you just pass in a file name it will compile it and execute it.

Run it with **--help** to see the available arguments.

You can exit by typing **exit**


## Tests
```
cargo test
```

## Coverage
```
cargo tarpaulin
```
If you want to see the coverage report via html:
```
cargo tarpaulin --out HTML
```

# Language features
It supports the following values: 
- numbers 
- strings 
- booleans

### Variable declarations
``` lua
var a = 3;
var b = "Hello, World!";
var c = true
var d;

print a;
print b;
print c;
print d;

// Prints
3
Hello, World!
true
nil
```
### Binary operators
It supports the following arithmetic binary operators:
- addition (+)
- subtraction (-)
- multiplication (*)
- division (/)
- modulo (%)

``` lua
print 6 + 2;
print 6 - 2;
print 6 * 2;
print 6 / 2;
print 6 % 2;


// Prints
8
4
12
3
0
```

``` lua
// You can also concatenate any value to a string
print "Hello, " + "World!";
print "Number: " + 3;
print "Nil: " + nil;
print "Boolean: " + true;

// Prints
Hello, World!
Number: 3
Nil: nil
Boolean: true

```
It supports the following logical binary operators:
- and
- or

``` lua
print true and true;
print true and false;
print false and false;
print true or true;
print true or false;
print false or false;

// Prints
true
false
false
true
true
false
```
It supports the following comparison binary operators:
- equal (==)
- not equal (!=)
- greater than (>)
- greater than or equal (>=)
- less than (<)
- less than or equal (<=)

``` lua
print 3 == 3;
print 3 != 3;
print 3 > 3;
print 3 >= 3;
print 3 < 3;
print 3 <= 3;

// Prints
true
false
false
true
false
true
```

It supports the following unary operators:
- negation (-)
- not (!)

``` lua
print -3;
print !3;
print !!3;

// Prints
-3
false
true
```

## Loops
While loop:
``` lua
var a = 0;
while (a < 5) {
    print a;
    a = a + 1;
}

// Prints
0
1
2
3
4
```

for loop:
``` lua
for (var i = 0; i < 3; i = i + 1) {
    print i;
}


var i;
for (i = 0; i < 3; i = i + 1) {
    print i;
}

var i = 0;
for (; i < 3; i = i + 1) {
    print i;
}

var i = 0;
for (; i < 3;) {
    print i;
    i = i + 1;
}

// All these loops print the same thing
0
1
2
```

## Functions
``` lua
fun add(a, b) {
    return a + b;
}

print add(3, 4);

// Prints
7
```

``` lua
fun print_hello() {
    print "Hello, World!";
}

print_hello();

// Prints
Hello, World!
```

``` lua
fun no_ret() { }
print no_ret();

// Prints
nil
```

``` lua
fun fib (n) {
    if (n < 2) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

fib(10);

// Prints
55
```

It currently has only a single native function (which calls rust code):
``` lua
print clock();

// Prints the number of seconds since the epoch
```
The **print** function is built in and is not considered a native function.


## Scopes and locals
``` lua
var a = 3;
{
    var b = 4;
    var a = 5;  // Shadows the outer variable
    print a;
    print b;
}               // 'b' is no longer in scope and the inner 'a' is no longer in scope
print a;

// Prints
5
4
3
```

## References
- [Crafting Interpreters](https://craftinginterpreters.com/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust std](https://doc.rust-lang.org/std/)
- [Darksecond Lox](https://github.com/Darksecond/lox)
- [x86 instruction set](https://c9x.me/x86/)