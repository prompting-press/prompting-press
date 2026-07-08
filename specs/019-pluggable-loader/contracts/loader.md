# Contract: PromptLoader (per binding)

Uniform capability; native idiom (C-06). Three parallel ecosystems (a loader is language-local).

## Rust (`prompting-press` consumer)
```rust
pub trait PromptLoader: Send + Sync {
    fn load(&self, key: &str) -> Result<String, LoadError>;   // raw text
}
impl<F> PromptLoader for F where F: Fn(&str) -> Result<String, LoadError> + Send + Sync { /* blanket */ }

pub struct FileSystemLoader { /* base: PathBuf, suffix: String (default ".yaml") */ }
pub struct MemoryLoader { /* HashMap<String,String> */ }
// FileSystemLoader::load rejects keys escaping `base` with LoadError (traversal guard).
// compose: let p = Prompt::from_yaml(&loader.load("bull_v1")?)?;
```

## Python (`prompting-press` wheel)
```python
@runtime_checkable
class PromptLoader(Protocol):
    def load(self, key: str) -> str: ...

class FileSystemLoader:
    def __init__(self, base: str | Path, suffix: str = ".yaml"): ...
    def load(self, key: str) -> str: ...   # raises LoadError; traversal-guarded
class MemoryLoader:
    def __init__(self, prompts: dict[str, str]): ...
    def load(self, key: str) -> str: ...
# callable coercion: any `Callable[[str], str]` works as a loader.
# compose: p = Prompt.from_yaml(loader.load("bull_v1"))
```

## TypeScript (`prompting-press` npm)
```ts
export interface PromptLoader { load(key: string): Promise<string>; }   // async (node fs / cloud)
export class FileSystemLoader implements PromptLoader { /* base, suffix=".yaml" */ }
export class MemoryLoader implements PromptLoader { /* Record<string,string> */ }
// function coercion: (key) => Promise<string> works as a loader.
// compose: const p = Prompt.fromYaml(await loader.load("bull_v1"));
```

## Cross-binding contract
- `load(key)` returns raw text; never a Prompt, never parses.
- Missing key / traversal escape → `LoadError` (normalized `[{field,code,message}]`), distinct from parse errors.
- FileSystemLoader: `{base}/{key}{suffix}`, traversal-guarded. MemoryLoader: key→text map.
- Custom loader = implement the interface (or pass a callable/function); NO registration.
- NOT fused into construction; NO name-keyed container in this feature.
