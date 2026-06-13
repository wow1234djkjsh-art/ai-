# C-DSL

C-DSL (Compact DSL) is a lightweight scripting language built in Rust, designed for AI-powered automation. Its space-call syntax and built-in `model` function make it easy to write scripts that talk to Claude and process the results.

---

## Quick Start

```bash
cargo build --release
./target/release/c-dsl --run script.dsl
```

### REPL

```bash
./target/release/c-dsl
> print "hello, world"
hello, world
```

---

## Claude API Integration

### 1. Set your API key

Get a key from [console.anthropic.com](https://console.anthropic.com) and set it as an environment variable:

```powershell
# Windows (PowerShell)
$env:ANTHROPIC_API_KEY = "sk-ant-api..."
```

```bash
# macOS / Linux
export ANTHROPIC_API_KEY="sk-ant-api..."
```

### 2. Call `model`

```
model <model-id> <prompt>
```

The result is a `String` containing Claude's response. Responses are cached on disk (`~/.c-dsl/cache/`) so repeated calls with the same model+prompt are instant and free.

```
answer = model "claude-opus-4-8" "What is the capital of France?"
print answer
```

**With format hint** — passing `"code"` as a third argument strips markdown fences from the response:

```
code = model "claude-opus-4-8" "Write a Python function to reverse a string" "code"
print code
```

**Force fresh call** — bypass cache with a fourth arg `"true"`:

```
result = model "claude-opus-4-8" "Explain entropy" "" "true"
```

**Available model IDs:**

| Model | ID |
|---|---|
| Claude Opus 4.8 | `claude-opus-4-8` |
| Claude Sonnet 4.6 | `claude-sonnet-4-6` |
| Claude Haiku 4.5 | `claude-haiku-4-5-20251001` |

### 3. Batch processing with `map`

```
topics = ["photosynthesis", "entropy", "recursion"]
answers = map(topics, fn t => model "claude-haiku-4-5-20251001" "Explain briefly: " + t)
each answers : fn a => print a
```

---

## Language Reference

### Variables

```
x = 42
name = "Alice"
```

### Numbers and Arithmetic

```
x = 10 + 3 * 2    // 16
y = x / 4          // 4
```

### Strings

```
s = "hello\nworld"     // \n \t \r \\ \" \0 supported
msg = "Hello, " + name
```

### Comments

```
// This is a line comment
# This also works (shell style)
```

### Conditionals

```
? x > 5 : print "big" : print "small"
```

The ternary `? cond : then : else` returns the value of the taken branch.

### Functions

```
fn square x => x * x
fn add a, b => a + b

square 9          // 81
add 3, 4          // 7
```

**Inline lambdas:**

```
double = fn x => x * 2
double 5          // 10
```

### Each (for-each loop)

```
each 1, 2, 3 : fn x => print x
each myList  : fn x => print x
```

### While loop

```
i = 0
while i < 5
  print i
  i = i + 1
end
```

### Pipe operator

```
"hello world" | upper | trim
3 | double | double    // 12
```

### Lists

```
nums = [1, 2, 3, 4, 5]
nums[0]               // 1
len(nums)             // 5
push(nums, 6)         // [1, 2, 3, 4, 5, 6]
first(nums)           // 1
last(nums)            // 5
pop(nums)             // [1, 2, 3, 4]
concat(nums, [6, 7])  // [1, 2, 3, 4, 5, 6, 7]
flat([[1, 2], [3]])   // [1, 2, 3]
slice(nums, 1, 3)     // [2, 3]
sort(nums)
contains(nums, 3)     // 1 (true)
range(5)              // [0, 1, 2, 3, 4]
range(2, 6)           // [2, 3, 4, 5]
```

### Dicts

```
person = {name: "Alice", age: 30}
person["name"]           // "Alice"
person.age               // 30
keys(person)             // ["name", "age"]
values(person)           // ["Alice", 30]
set(person, "age", 31)   // new dict with age updated to 31
```

### Higher-order functions

```
doubled = map([1, 2, 3], fn x => x * 2)
evens   = filter([1, 2, 3, 4], fn x => x % 2 == 0)
sum     = reduce([1, 2, 3, 4], fn acc x => acc + x, 0)
```

### Try / Catch

```
try
  result = model "bad-model-id" "hello"
  print result
catch e
  print "Error: " + e.message
end
```

### String functions

```
split("a,b,c", ",")   // ["a", "b", "c"]
split("hello world")  // ["hello", "world"]
join(["a", "b"], "-") // "a-b"
upper("hello")        // "HELLO"
lower("HELLO")        // "hello"
trim("  hi  ")        // "hi"
```

### Type conversion

```
str(42)        // "42"
num("3.14")    // 3.14
type(42)       // "number"
type([])       // "list"
is_nil(nil)    // 1
```

### Nil

Missing dict keys return `nil`. You can test for it:

```
d = {x: 1}
is_nil(d["y"])       // 1
d["y"] == nil        // 1  (nil == nil is true)
d["y"] == d["x"]     // 0  (nil != number)
```

### Math

```
floor(3.7)   // 3
ceil(3.2)    // 4
round(3.5)   // 4
abs(-5)      // 5
min(3, 7)    // 3
max(3, 7)    // 7
```

### File I/O

```
content = read_file("data.txt")
write_file("out.txt", "hello\n")
append_file("log.txt", "new line\n")
```

### HTTP and JSON

```
resp = http_get("https://api.example.com/data")
data = json_parse(resp)
print data["key"]

json_str({name: "Alice", score: 99})  // '{"name":"Alice","score":99}'
```

### Process

```
sleep(500)   // wait 500 ms
exit(0)      // exit with code 0
```

### I/O

```
name = input("Enter your name: ")
print "Hello, " name     // multi-arg print joins with space
```

### Environment variables

```
key = env("ANTHROPIC_API_KEY")
```

### CLI arguments

Extra arguments passed after the script path are available as the `args` list:

```bash
c-dsl --run script.dsl foo bar
```

```
// script.dsl
print first(args)   // "foo"
print args[1]       // "bar"
```

### Eval

```
code = "1 + 2"
result = eval(code)   // 3
```

---

## Practical Examples

### Summarize a list of topics

```
topics = ["Rust ownership", "monads", "Fourier transform"]
each topics : fn topic =>
  summary = model "claude-haiku-4-5-20251001" "Explain in one sentence: " + topic
  print topic + " → " + summary
end
```

### Generate and run code

```
code = model "claude-opus-4-8" "Write a C-DSL expression for the sum of 1+2+3+4+5" "code"
result = eval(code)
print "Result: " + str(result)
```

### Classify items

```
items = ["apple", "carrot", "banana", "broccoli"]
fn classify item =>
  model "claude-haiku-4-5-20251001" "Reply with only 'fruit' or 'vegetable': " + item

fruits = filter(items, fn x => classify(x) == "fruit")
print join(fruits, ", ")
```

### Read → process → write pipeline

```
raw = read_file("input.txt")
lines = split(raw, "\n")
processed = map(lines, fn line =>
  model "claude-haiku-4-5-20251001" "Fix grammar: " + line
)
write_file("output.txt", join(processed, "\n"))
```

### Retry with error handling

```
fn ask prompt =>
  try
    model "claude-sonnet-4-6" prompt
  catch e
    "Error: " + e.message
  end

result = ask "What is 2+2?"
print result
```

---

## Running Tests

```bash
cargo test
```
