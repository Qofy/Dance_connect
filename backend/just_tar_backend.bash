#!/usr/bin/env bash

set -ueE

function main() {

  local -i _err=0
  cd ../20260123_mitote_your_culinary_rust_v025  || (echo "failed to go to backend" && exit 1) || exit 1
  # cd ../backend || echo "failed to go to backend" && exit 1
  _err=$?
  if [ ${_err} -gt 0 ] ; then
  {
      echo "cd into backend Has Errors"
      exit 1
  }
  else
  {
     echo "cd into backend No Errors"
     pwd
  }
  fi

  local excluding=""
  excluding="$(ls -1 | grep -Ev 'klirr|build.sh|build.rs|src|Cargo.toml')
  .dir_bash_history
backend.tar.gz
.git
.env_cors
.env
src/_3_models3.rs
src/_3_models3_chop.rs
.vscode
.temp_keys
.claude
"
  local excludes=""
  local one=""
  while read -r one; do
  {
    [[ -z "${one}" ]] && continue
    excludes="${excludes} --exclude ${one}"
  }
  done <<< "${excluding}"
  tar -czf ../backend.tar.gz --show-omitted-dirs --preserve-permissions ${excludes} . 2>&1
  _err=$?

  if [ ${_err} -gt 0 ] ; then
  {
      echo "Zipping Has Errors"
      echo "FAILED Zips:
  ---------------- zipping_failed >>>
  ---------------- <<< zipping_failed "
      echo "Finished zipping Has Errors"
      exit ${_err}
  }
  else
  {
     echo "Finished zipping No Errors"
  }
  fi

} # end main

main
