# C-DSL Syntax Reference

AI-native scripting language. Minimal tokens, optional whitespace, semicolons or newlines as separators.

---

## Literals

```
42
3.14
"hello"
```

---

## Variables

```
x=5
y="hello"
z=x+3
```

---

## Arithmetic

```
x+y    x-y    x*y    x/y
```

Zero division returns `nil`.  
String concat: `"a"+"b"` → `"ab"`

---

## Comparison

Returns `1` (true) or `0` (false).

```
x>y    x<y
```

---

## Conditionals

`?cond:then:else` — all three parts required.

```
?x>0:x:-x
?x>10:"big":"small"
?x>0:x*2:0
```

Nested:

```
?x>10:"big":?x>0:"mid":"small"
```

---

## Functions

Define:

```
fn add a,b=>a+b
fn double x=>x*2
fn abs x=>?x>0:x:0-x
```

Call (space syntax):

```
add 1,2
double 5
```

Call (paren syntax):

```
add(1,2)
double(5)
abs(-3)
```

Recursive:

```
fn fact n=>?n>0:n*fact n-1:1
fact 5    → 120
```

Closures (captures outer variables):

```
x=10
fn add_x n=>n+x
add_x 5    → 15
```

---

## Pipes

Left result is passed as first argument to the right.

```
3|double           → 6
3|double|double    → 12
add 1,2|double     → 6
5|print            → prints 5, returns 5
```

---

## Each (iteration)

Apply function to each item, returns last result.

```
each 1,2,3:fn x=>x*2    → 6
each 1,2,3:fn x=>?x>1:x*2:x    → 6
```

---

## Builtins

| Name | Usage | Behavior |
|------|-------|----------|
| `print` | `print x` or `x\|print` | Prints value, returns it unchanged |
| `eval` | `eval "x+1"` | Evaluates a C-DSL string |
| `model` | `model "id","prompt"` | AI model call (stub) |

---

## Statement Separator

`;` or newline — both work.

```
x=3;y=4;add x,y
x=3
y=4
add x,y
```

---

## Full Examples

```
fn double x=>x*2
x=5
?x>3:x|double:0|print
```

```
fn fact n=>?n>0:n*fact n-1:1
fact 10
```

```
each 1,2,3,4,5:fn x=>?x>2:x*x:x
```
