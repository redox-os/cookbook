# Important note for libraries!

You might be tempted to convert recipes from:

```toml
[build]
template = "custom"
script = """
DYNAMIC_INIT
cookbook_cmake
"""
```

to this:

```toml
[build]
template = "cmake"
```

But that is incorrect! All library templates should keep using `custom` script with `DYNAMIC_INIT` inside.

## How cookbook templates and dynamic linking works

Other than `custom`, cookbook has 4 kind of templates: `cargo`, `configure`, `cmake`, `meson`. These templates implicitly add `DYNAMIC_INIT` only if build depedencies > 0.

The reason `DYNAMIC_INIT` is not automatically added into `custom` or build deps == 0 is to make them silently compile as statically linked binaries, which is a must for drivers and init systems to avoid them being linked to `libgcc`, `libstdcxx` or `relibc` as it's not available for them during system bootstrap. Unfortunately, this rule is also applies for libraries.

If you're not adding `DYNAMIC_INIT` for libraries, these libraries will only emit static libraries (`.a` files) which will NOT work for binaries that uses `DYNAMIC_INIT`!

In short, only use `cargo`, `configure`, `cmake`, `meson` template for binaries!
