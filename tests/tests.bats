#!/usr/bin/env bats

bats_require_minimum_version 1.5.0

bats_load_library bats-support
bats_load_library bats-assert
bats_load_library bats-file

# Make all internal paths relative to the test directory
export HOME=${BATS_TEST_DIRNAME%/.}

setup() {
  cd "$BATS_TEST_DIRNAME" || exit 1
  rm -rf .config work_dir
  mkdir -p work_dir
  cd work_dir || exit 1
}

@test "--help" {
  run shdw --help
  assert_success
}

@test "shadow-dir should exist" {
  run shdw --shadow-dir=non-existent ls
  assert_failure
}

@test "add file" {
  touch file
  shdw add file
  run readlink file
  assert_output ../.config/shdw/dir/work_dir/file
}

@test "adding non-existend file should fail" {
  run ! shdw add non-existent
}

@test "remove file" {
  touch file
  [[ ! -L file ]]
  shdw add file
  [[ -L file ]]
  shdw rm file
  [[ ! -L file ]]
}

@test "add empty directory" {
  mkdir dir
  run shdw add dir
  assert_success
}
