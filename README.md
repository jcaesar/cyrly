# cyrly

A serde-based YAML serializer for Rust with an unusual output style.

## Usage

```rust,ignore
cyrly::to_string(some_val)?
```
will prdouce something like
```yaml
{
  { 42: 1336 }: "non-string keys",
  "three different string styles": [
    plain,
    "single-line strings",
    "\
      multi-line strings are\n\
      acceptably readable\n\
      \n\
      also, yaml 1.1/1.2 ambiguities are quoted:
    ",
    "oFf",
  ],
  look: "trailing comma",
}
```

## Misc

I think this is better than X because Y:
 * JSON
   * Fewer `"`
   * Trailing commas,
   * Multiline  
     strings
   * Support for non-string keys
 * YAML (as produced by normal serializers)
   * Not whitespace-indentation dependent
 * JSON5 / Hjson
   * Valid YAML, which is a much more common format, thus the output will be usable in many more places

Note that while the serializer attempts to be conservative where possible
(e.g. always quoting maybe-keywords like `no` or `on`)
and should always produce valid YAML,
it is still somewhat unusual and may trouble some YAML deserialization implementations.