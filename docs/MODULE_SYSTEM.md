# Module System Design

## Syntax

### Import Statements

```aether
// Import entire module
import math

// Import with alias
import math as m

// Import specific items
from math import abs, min, max

// Import specific item with alias
from math import abs as absolute
```

## Module Resolution

### File Structure
```
project/
├── main.ae           # Entry point
├── utils.ae          # User module
└── modules/
    └── helper.ae     # Nested module
```

### Resolution Rules
1. Look for `<name>.ae` in current directory
2. Look for `modules/<name>.ae` relative to current file
3. Look for `<name>.ae` in stdlib (embedded modules)

### Module Path
- Relative to importing file's directory
- No nested imports yet (e.g., `import utils.helper` not supported in Phase 1)

## Implementation Plan

### Phase 1: Basic Import (Current Sprint)
- `import module` - loads `module.ae`
- `from module import func` - imports specific functions
- Module caching to prevent circular dependencies
- Namespace isolation (modules don't pollute global scope)

### Phase 2: Advanced Features (Future)
- Nested modules: `import utils.helper`
- Directory modules with `__init__.ae`
- Relative imports: `from . import sibling`
- Export control: `__all__` or explicit exports

## Namespace Management

### Global Environment
```
main.ae environment:
  - All main.ae definitions
  - Imported modules as objects
```

### Module Environment
```
module.ae environment:
  - Module's own definitions
  - Isolated from importing file
```

### Access Pattern
```aether
import math

math.abs(-5)    // Access via module namespace
```

```aether
from math import abs

abs(-5)         // Direct access (imported into current scope)
```

## Circular Dependency Handling

### Strategy: Module Caching
1. Track modules being loaded (loading_stack)
2. Track loaded modules (module_cache)
3. If module is in loading_stack, return partially loaded state
4. Once loaded, cache the module

### Example
```aether
// a.ae
import b
fn a_func() { return b.b_func() }

// b.ae
import a
fn b_func() { return a.a_func() }  // Error or partial load
```

**Resolution**: Detect cycle and error or allow partial loading.

## Implementation Details

### AST Changes
```rust
enum Stmt {
    // ...
    Import(String),                           // import module
    ImportAs(String, String),                 // import module as alias
    FromImport(String, Vec<String>),          // from module import a, b, c
    FromImportAs(String, Vec<(String, String)>), // from module import a as x
}
```

### Module Loader
```rust
struct ModuleLoader {
    module_cache: HashMap<String, Environment>,
    loading_stack: Vec<String>,
    search_paths: Vec<PathBuf>,
}
```

### Evaluator Changes
```rust
impl Evaluator {
    pub fn load_module(&mut self, path: &str) -> Result<Environment, RuntimeError>
    pub fn exec_import(&mut self, stmt: &ImportStmt) -> Result<(), RuntimeError>
}
```

## Testing Strategy

### Test Cases
1. Basic import (import module)
2. Import with alias (import module as m)
3. From import (from module import func)
4. Module not found error
5. Circular dependency detection
6. Namespace isolation
7. Multiple imports of same module (caching)
8. Accessing module members

### Test Files
```
tests/modules/
├── basic_module.ae      # Simple module with functions
├── user_module.ae       # Test user-defined modules
└── circular_a.ae        # Circular dependency test
    circular_b.ae
```

## Error Messages

```
Error: Module not found: 'unknown'
Error: Circular dependency detected: a -> b -> a
Error: Module 'math' has no function 'unknown_func'
Error: Cannot import 'x' from module 'math'
```

## Limitations (Phase 1)

1. No nested module paths (`utils.helper`)
2. No directory modules
3. No relative imports (`.`, `..`)
4. No export control (everything is exported)
5. No dynamic imports (runtime string paths)
6. File-based only (no in-memory modules except stdlib)

## Future Enhancements

1. Package system with `package.ae` manifest
2. Version management
3. Remote module fetching
4. Module precompilation/caching
5. Lazy loading (on first use)
