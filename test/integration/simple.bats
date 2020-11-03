#!/usr/bin/env bats

load "helpers"

@test "Execute simple mdiff" {
  cmd `fixture`/first.yml `fixture`/second.yml
}
