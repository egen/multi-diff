REPO_ROOT=$(git rev-parse --show-toplevel)

cmd() {
  mdiff=${REPO_ROOT}/target/debug/multi-diff
  if [[ ! -z ${CARGO_TARGET_DIR} ]] ; then
    mdiff=${CARGO_TARGET_DIR}/debug/multi-diff
  fi

  ${mdiff} $@
}

fixture() {
  echo "test/fixtures/$(basename ${BATS_TEST_FILENAME%%.*})"
}
