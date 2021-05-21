#! /bin/bash
set -e
test -n "${EXAMPLE1_DIR}"
test -n "${EXAMPLE1_VALUE}"
set -x

cat <<EOF > "${EXAMPLE1_DIR}/example1.conf"
example1=${EXAMPLE1_VALUE}
EOF
chmod 640 "${EXAMPLE1_DIR}/example1.conf"

