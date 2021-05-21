#! /bin/bash
set -e
test -n "${EXAMPLE1_DIR}"
test -n "${EXAMPLE1_VALUE}"
set -x
test -f "${EXAMPLE1_DIR}/example1.conf"
