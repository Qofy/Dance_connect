#!/usr/bin/env bash
set -ue

function main() {

  local -i _err=0
  local msg=""
  pwd
  # ls -la ./"backend.tar.gz"
  # quote
  # msg=$( knockscp.sh velotermin.bike velotermin_bike ./"backend.tar.gz" "/home/zeus/_/software/mitote.intuivo.com/backend.tar.gz"  2>&1);
  # faktura
  # msg=$( knockscp.sh velotermin.bike velotermin_bike ./"backend.tar.gz" "/home/zeus/_/software/faktura.intuivo.com_backend/backend.tar.gz"  2>&1);
  _err=$?

  if [ ${_err} -gt 0 ] ; then
  {
      echo "un- Finished pushing Has Errors: ${_err} "
      echo "FAILED UPLOADS:
      ---------------- uploads_failed >>>
       knockscp.sh velotermin.bike velotermin_bike ./"backend.tar.gz" "/home/zeus/_/software/faktura.intuivo.com_backend/backend.tar.gz"
      >>>>>>>>>>>> ------
      ${msg}
      ---------------- <<< uploads_failed: ${_err}  "
      echo "un -Finished pushing ...because Has Errors"
      exit ${_err}
  }
  fi
  echo "Finished pushing = YAY! No mistakes "
  function main_ejecutar() {
    local service_url="https://faktura.intuivo.com"
    local passwords="--proxy-user jesusalc:machoman --user jesusalc:machoman "
    set -x
    #
    # quote
    # msg="$(curl $passwords -k $service_url/pass_backend_to_server_quote.php)"
    # faktura
    msg="$(curl $passwords -k $service_url/faktura_backend_clone_compile.php)"
    _err=$?
    set +x

    if [ ${_err} -gt 0 ] ; then
    {
        echo "Trigger unzip Has Errors: ${_err} "
        echo "UNZIP ERRORS :
        ---------------- unzip_failed >>>
        ${msg}
        ---------------- <<< unzip_failed: ${_err}  "
        echo "un finished unzunip ..  Has Errors"
        exit ${_err}
    }
    else
    {
       echo "Finished remove unzipping = YAY! No mistakes "
       echo "${msg}"
    }
    fi

  } # end main_ejecutar

  main_ejecutar



} # end main

main
