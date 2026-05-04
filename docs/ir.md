# Tibanna IR

Tibanna's IR is:
 - a control flow graph
 - in SSA form

## High Level Structure
A program is semantically represented as a collection of functions
> Program => [Function]*

A function is composed of a set of `Block`s. Control flow is explicit through jumps/branches.

---
### Blocks
```rust
Block {
  label: BlockId,
  intructions: [Instruction]
  terminator: Terminator
}
```

A block is a sequence of instructions. Each block ends with exactly one terminator and has no implicit fallthrough

---
### SSA Form
- Each value is assigned exactly once
- Every assignment results in a new SSA value
- Source variables are all lowered into SSA temps

Example:
```
let x = 1;
x = x * 2;
```
Becomes:
```
_v0 = const_int 1
_v1 = mul _v0, 2
```

### Instructions
#### Constants
```
_v0 = const_int 420
_v1 = const_bool true
```

#### Binary Operations
```
_v2 = add _v0, _v0
_v3 = sub _v2, _v0
_v4 = mul _v0, _v2

```
**Comparisons**
```
_v5 = lt _v0, _v2
_v6 = eq _v0, _v2
```

**Logical**
```
_v7 = and _v1, _v1
_v8 = or _v1, _v7
```

### Function Calls
```
_v9 = call foo(_v0, _v1)
```

### Terminators
Each block must end with exactly one terminator
#### Unconditional Branch
```
br block_id(...)
```
#### Conditional Branch
```
br_cond <cond>, then_block(...), else_block(...)
```

### Phi Nodes
Tibanna IR avoid explicit phi instructions by using block parameters

Instead of:
```
_v10 = phi [_v0, B0], [_v1, B1]
```
Becomes
```
merge_block(...)
```
With predecessors:
```
B0:
  br merge_block(_v0)

B1:
  br merge_block(_v1)
```

### Example
#### Tibanna Source
```
fn foo(a: int, b: int) = int {
  let x = a + b;
  if x > 0 {
    x = x - 1;
  } else {
    x = x + 1;
  }
  return x;
}
```
#### T-IR
```
fn foo(a, b) {
  entry:
    _v0 = add a, b
    _v1 = const_int 0
    _v2 = gt _v0, _v1
    br_cond _v2, then_block(_v0), else_block(_v0)
  
  then_block(x_then):
    _v3 = const_int 1
    _v4 = sub x_then, _v3
    br merge_block(_v4)
  
  else_block(x_else):
    _v5 = const_int 1
    _v6 = add x_else, _v5
    br merge_block(_v6)
  
  merge_block(x_final):
    return x_final
}
```
