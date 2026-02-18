#!/bin/bash

function main() {
  set -xu
  set +eE
  local -i _err=0
  clear
  echo "cargo test  ${*-}"
  cargo test  ${*-}
  _err=$?
  [[ ${_err} -gt 0 ]] && echo "clipping" &&  .   ./xtclipit.bash && exit ${_err}

}

main

#
