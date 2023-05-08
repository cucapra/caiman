# Caiman Frontend Summer 2023

The goal of this writing is to provide a concrete proposal and timeline for
summer work on the Caiman frontend.  Note that the details and plan is flexible,
but just to have something to reference.

## State of Affairs

The current Caiman frontend has standard value-language syntax and work is being
done on a "baseline translation".  Concretely, from what I understand, we can
write programs of the (rough) form:

```
// cavl
x := 5;
z := (- y x);
y := (if x > 3 (+ x 1) 0)

// casl
x;
y;
z;
```

Programs like this are (nearly) at the point of compiling and running; this
program should compile in such a way as to produce 1, for example.

## Summer Goals

I would propose the following "definite" goals at a high level.  For each, I'll
later propose what a sample program might look like to demonstrate the feature
"in action".

* Finish translation + testing setup
* External function "hooks" (e.g. basic arithmetic)
* Functions with straight-line calls
* Scheduling control flow

## Timeline

I would propose the following timeline for summer, just to have a concrete
proposal.  This came out to look like about 16 weeks, so we might need to cut
scope, depending on how things are going.  Note that times are flexible and
uncertain, but just to have _something_:

* Cleanup initial pass based on "state of affairs" (1-3 weeks?)
* Get testing setup working (1 week)
* Nail down syntax + write out "example programs" carefully (1 week)
* External function hooks (2-3 weeks)
* Functions (1-2 weeks)
* Scheduling control flow through explicit calls (2-3 weeks)
* Rest of scheduling control flow (3-4 weeks)
* Finalize recursive example, both versions (1-2 weeks)
* Future plans + writeup (1-2 weeks)

# Goal Details

For each of the goals above, let's take a look at a couple of "target" programs.
There are more that could be written of course, and writing more examples seems
like a useful idea.  I will also note that the syntax is very preliminary, so
feel free to workshop it, though I think it would make sense at this point to
propose changes so that we can reach a team agreement on syntax.

## Translation/Testing

This is the one exception to having an example, as I think the current programs
are reasonable to demonstrate things.  If the above program works and compiles,
then this "stage" should be considered a success.

The main goal here should probably be to hook up the existing frontend to the
Caiman compiler frontend setup, both to help with testing and to allow easier
integration of future work and features.

## External "hooks"

Externals might already be setup, but I think it's still worth defining a few
simple examples to make sure the syntax/setup is on the same page.  Otherwise
none of the other examples really make sense.  Note that I assume fairly simple
scheduling still, I think it's worth avoiding until functions are up and running
(described below).

Here is an examples I would consider to be the goal here.  Note that `result` is
the name of the variable that we are computing, for the purpose of having a
"correct output".

```
//cavl
result := (- (- z y) (- x y));
z := (* x y).
x := 3.
y := (+ x 1).

//casl
x;
y;
z;
result;

// Expected: 9
```

## Functions

The most obviously useful thing we need is the ability to write value and
associated schedule functions.  These are not funclets, they are true functions
(that is, control flow will be permitted within the schedule function bodies).
Formally, allowing functions consists of the following three things:

1. Function headers (with typed arguments and returns)
2. Value function "equal sets" per Caiman requirements
3. The ability to call functions by name (note that typechecking is done later)

Here are a couple of programs that "set this up", at least in my head.

Argument (with allocation and operation):

```
value main(x : i64) -> i64 {
    // temporary syntax for naming order
    result := returns (+ x 1).
}

schedule main with main_s(x_loc : *i64) -> *i64 {
    result;
}

// main(5) -> 6, main(0) -> 1
```

Function call:

```
value main(x : i64) -> i64 {
    y := 5.
    calc_left := foo(x, y).
    calc_right := foo(y, x).
    result := returns (+ calc_x calc_y).
}

value foo(x : i64, y : i64) -> i64 {
    result := returns (+ (- x y) 1).
}

schedule main with main_s(x_loc : *i64) -> *i64 {
    y;
    calc_left;
    calc_right;
    result;
}

schedule foo with foo_s(x_loc : *i64, y_loc : *i64) -> *i64 {
    // just do all the things?
    // could rely on explication!
    result;
}
```

Recursion (this won't actually run of course):

```
value rec() -> i64 {
    rec_result := (rec). // is this a call?
    result := returns rec_result.
}

schedule main with main_s(x_loc : *i64) -> *i64 {
    rec_result;
    result;
}
```

## Scheduling Control Flow

So far we've been using "hacky" scheduling syntax to just say "do this whole
line, and let the explicator figure it out".  We want to be able to break this
down, both for functions and conditions.  In particular, we need the "full
syntax" for the following:

1. Explicit holes
2. Allocation
3. Explicit function calls + assignment
4. Built-in function calls
5. Tail edges (returns)
6. Conditionals

As usual, we'll look at an example program for each to try and get something
more "iterative".

### Explicit Hole

```
value main(x : i64) -> i64 {
    returns (+ x 1).
}

schedule main with main_s(_ : *i64) -> *i64 {
    ???
}
```

### Allocation + Constant

```
value main() -> i64 {
    x := 3.
    returns x.
}

schedule main with main_s() -> *i64 {
    // we can infer the associated variable on transformation?
    // specifically by letting the explicator deal with it!
    allocate x_loc; // local variable
    allocate x_const;
    x_loc = call $x(x_const); // explicit
    ??? // call + return stuff
}
```

### Explicit Call

```
value main(x : i64) -> i64 {
    y := 5.
    calc_left := foo(x, y).
}

value foo(x : i64, y : i64) -> i64 {
    result := returns (+ x y).
}

schedule main with main_s(x_loc : *i64) -> *i64 {
    allocate y_loc; // is this reasonable?
    y_loc <- call $y(?); // constant hole
    allocate calc_left_loc;
    calc_left_loc <- call $calc_left(x_loc, y_loc);
    ??? // return
}

schedule foo with foo_s(_ : *i64, _ : *i64) -> *i64 {
    ???
}
```

### Built-in Function Calls

```
value main(x : i64, y : i64) -> i64 {
    result := (+ (- x y) 1).
    returns result.
}

schedule main with main_s(x_loc : *i64, y_loc *i64) -> *i64 {
    // I think there was better syntax here, but I'm forgetting
    allocate sub_loc;
    sub_loc <- call $result.0(x_loc, y_loc);

    allocate const_loc;
    const_loc <- assign $result.1;
    call $result(sub_loc, const_loc);
    ??? // return
}
```

### Tail-Edges (Returns)

```
// this can be used to update previous examples of course!
value main() -> i64 {
    x := 3.
    returns x.
}

schedule main with main_s() -> *i64 {
    allocate x_loc;
    x_loc <- call $x(?);

    return x_loc;
}
```

Also I like this syntax

```
value main(x : i64, y : i64) -> i64 {
    returns (+ (- x y) 1).
}

schedule main with main_s(x_loc : *i64, y_loc *i64) -> *i64 {
    return {
        allocate sub_loc;
        sub_loc <- call $returns.0(x_loc, y_loc);
        const_loc = allocate $returns.1.const;
        call $returns(sub_loc, const_loc) // no `;` means return
    }
}
```

### Conditionals (Plus Recursion!)

```
value sum(x : i64) -> i64 {
    left := (sum (- x 1)).
    left_res := (+ x left).
    result := (if (= x 0) left_res 0).
    returns result.
}

schedule sum with sum_sch(x_loc : *i64) -> *i64 {
    allocate check;
    allocate zero;
    zero <- $result.0.1; // no call needed?
    check <- call $result.0(zero, x_loc);
    return { 
        $result.if(check) {
            allocate calc_loc; // the types work out?
            allocate one;

            one <- $left.0.1;

            // this is kinda cool
            calc_loc <- $left.0(x_loc, one);
            calc_loc <- $left(calc_loc);
            calc_loc <- $left_res(x, calc_loc);
            
            calc_loc // no `;` means return
        }
        else { 
            zero // I dunno if this makes type-sense
        }
    }
}
```

and with holes:

```
value sum(x : i64) -> i64 {
    left := (sum (- x 1)).
    left_res := (+ x left).
    result := (if (= x 0) left_res 0).
    returns result.
}

schedule sum with sum_sch(x : *i64) -> *i64 {
    ???
    return { 
        $result.if(?) {    
            ???
            $left.?;
            $left_res.?;
            ???
        }
        else { ??? }
    }
}
```