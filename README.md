# multi-diff

analyzes multiple yaml files and prints out common sub-trees:

```
$ cat test/fixtures/simple/first.yml
common_high_level:
  common_key: common
  shared_a: shared_a
  different_value:
    nested: value
  first: first

different_high_level: first
$ cat test/fixtures/simple/second.yml
common_high_level:
  common_key: common
  shared_a: shared_a
  shared_b:
    key: value
  different_value: second
  second: second

different_high_level: second
$ cat test/fixtures/simple/third.yml
common_high_level:
  common_key: common
  different_value:
    nested: value
  shared_b:
    key: value
  third: third
$ make build && ./target/debug/multi-diff test/fixtures/simple/first.yml test/fixtures/simple/second.yml test/fixtures/simple/third.yml
cargo build
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
# test/fixtures/simple/third.yml
# test/fixtures/simple/first.yml
# test/fixtures/simple/second.yml
---
common_high_level:
  common_key: common
# test/fixtures/simple/second.yml
# test/fixtures/simple/third.yml
---
common_high_level:
  shared_b:
    key: value
# test/fixtures/simple/first.yml
# test/fixtures/simple/third.yml
---
common_high_level:
  different_value:
    nested: value
# test/fixtures/simple/second.yml
# test/fixtures/simple/first.yml
---
common_high_level:
  shared_a: shared_a
# test/fixtures/simple/third.yml
---
common_high_level:
  third: third
# test/fixtures/simple/second.yml
---
different_high_level: second
common_high_level:
  different_value: second
  second: second
# test/fixtures/simple/first.yml
---
different_high_level: first
common_high_level:
  first: first
```
