# Diagrams fixture

A mermaid flowchart:

```mermaid
graph LR
 A --> B
```

A mermaid sequence diagram:

```mermaid
sequenceDiagram
 Alice->>Bob: Hi
```

A DOT graph:

```dot
digraph G { a -> b }
```

A graphviz-aliased DOT graph:

```graphviz
digraph H { x -> y }
```

A regular Rust code block:

```rust
fn main() {}
```

A broken mermaid block:

```mermaid
not actually mermaid syntax %%%
```
