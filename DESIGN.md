# Design

## Technique

Sequence of steps. Solves by solving them with just "is_solved" checks.
```rust
struct Technique(Vec<Step<P>>);

impl Technique{
    fn solve(&self, puzzle: P); // does steps in order
}
```

## Method

Something that takes options and generates a technique. It generates the steps too - so a same "step shape" can be adapted depending on the technique.

```rust
Roux(Roux::Options): Technique;
```

Memos should be, in some way, global, so that when a technique goes out of scope, the memo of its steps shouldn't.

Unclear if trait or type.

### Trait Method

```rust
#[non_exaustive]
struct Roux{
    fb: FirstBlockOptions,
    sb: SecondBlockOptions,
    cmll: CMLLOptions,
    lse: LSEBlockOptions,
}

trait Method: Default {
    fn to_technique(&self) -> Technique;
    fn new(&self) -> Self {
        self.default()    
    }
    fn solve(&self, puzzle: &mut P) -> P::Solution {
        self.to_technique().solve
    }
}


impl Method for Roux {
    fn to_technique(&self) -> Technique{
        let FB = match roux.fb{
        }
        ...
        
        Technique([FB, SB, CMLL, LSE].join())    
    }
}
```