# Mock fixtures (internal)

In-process `mock` adapter used by **unit tests and continuous integration**.
Not for operators and not the public getting-started path.

Product scripts live in [`../harnesses/`](../harnesses/).

```bash
# Contributors only -- exercised by `cargo test` / `make check`
medon run examples/mock/full_driver_surface.rhai --print
```

Method map for suite coverage: [`../DRIVER_SURFACE.md`](../DRIVER_SURFACE.md).
