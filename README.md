# cyrly

A serde-based YAML serializer for Rust with an unusual output format.

## Usage

```rust
cyrly::to_string(some_val)?
```
will prdouce something like
```yaml
{
  { 42: 1336 }: "non-string\u0020keys",
  "three\u0020different\u0020string\u0020styles": [
    plain,
    "single-line\u0020or\u0020\"dangerous\"\u0020stuff\u0020like",
    "multiline\nnicely\u0020readable",
    "oFf",
  ],
  look: "trailing\u0020comma",
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