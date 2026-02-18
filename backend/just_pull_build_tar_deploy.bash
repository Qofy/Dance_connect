#!/usr/bin/env bash
function main(){
  THISSCRIPTNAME=$0
  set -u
  function _trap_on_error(){
    local -ir __trapped_error_exit_num="${2:-0}"
    echo -e "\\n \033[01;7m*** 2 ERROR TRAP $THISSCRIPTNAME \\n${BASH_SOURCE[0]}:${BASH_LINENO[-0]} ${FUNC_NAME[1]:-}() \\n$0:${BASH_LINENO[1]} ${FUNC_NAME[2]:-}()  \\n$0:${BASH_LINENO[2]} ${FUNC_NAME[3]:-}() \\n ERR ...\033[0m  \n \n "
    echo ". ${1}"
    echo ". exit  ${__trapped_error_exit_num}  "
    echo ". caller $(caller) "
    echo ". ${BASH_COMMAND}"
    local -r __caller=$(caller)
    local -ir __caller_line=$(echo "${__caller}" | cut -d' ' -f1)
    local -r __caller_script_name=$(echo "${__caller}" | cut -d' ' -f2)
    awk 'NR>L-10 && NR<L+10 { printf "%-10d%10s%s\n",NR,(NR==L?"☠ » » » > ":""),$0 }' L="${__caller_line}" "${__caller_script_name}"

    # $(eval ${BASH_COMMAND}  2>&1; )
    # echo -e " ☠ ${LIGHTPINK} Offending message:  ${__bash_error} ${RESET}"  >&2
    exit ${__trapped_error_exit_num}
  } # end _trap_on_error

  trap  '_trap_on_error $0 "${?}" LINENO BASH_LINENO FUNCNAME BASH_COMMAND $FUNCNAME $BASH_LINENO $LINENO   $BASH_COMMAND'  ERR

  local command_string=""
  local -i _err=0
  local -i _pid=0
  local _ret=""
  function run_command(){
    # REF: https://stackoverflow.com/questions/9261397/how-can-i-get-both-the-process-id-and-the-exit-code-from-a-bash-script
    # { sh -c "$executable > $log 2>&1 &"'
    # echo $! > pidfile
    # echo   # Alert parent that the pidfile has been written
    # wait $!
    # echo $? > exit-status
    # ' & } | read
    _err=${?:-}
    command_string="${*-}"
    echo "${_err:-}:${_pid:-}"
    if [ ${_err} -gt 0 ] ; then
    {
      echo " ::: ${command_string-}  ::: command before this command <<${command_string}>> has errors ::: pid:${_pid} ::: err:${_err}"
      exit ${_err}
    }
    fi
    echo -e "\033[01;27m \\ \x08\033[01;45m_____\033[0m\033[01;217m running:\033[0m\033[035m ${command_string} \033[0m"
    ${command_string}
    _err=${?:-}
    # _ret="${?:-}:${!}"
    # echo ":: ret: ${_ret}"
    # _err="${_ret%%:*}"   # same as # _err=$(cut -d: -f1 <<<"${_ret}" ) ;   cut -d: -f1 <<<"${_ret}"  >/tmp/_pid_rust.err
    # _pid="${_ret#*:}"    # same as # _pid=$(cut -d: -f2 <<<"${_ret}" ) ;   cut -d: -f2 <<<"${_ret}"  >/tmp/_pid_rust.pid
    #  waitpid ${_pid}
    # _err=${?:-}
    echo "${_err:-}:${_pid:-}"
    if [ ${_err} -gt 0 ] ; then
    {
      echo -e "\033[01;7m :::\033[0m*** ${command_string-} \033[01;7m*** :::\033[01;217m*** has errors\033[01;7m*** :::\033[01;219m*** pid:\033[01;117m***${_pid}\033[01;7m*** :::\033[01;217m*** err:\033[0m***${_err}"
      exit ${_err}
    }
    fi
  } # end run_command

  PROJECT_PATH=$(realpath ".")
  PROJECT_NAME=$(basename "$PROJECT_PATH")


  # run_command "pnpm self-update"
  if [ -d .git ] ; then
  {
    run_command "git pull"
  }
  fi
  pwd
  run_command ./just_tar_backend.bash
  run_command pwd
  pwd
  run_command ./just_upload_backend.bash

} # end main

main


#
