#!/usr/bin/env bash

# ------- #
# Globals #
# ------- #

DRY_RUN=0

# ------------------------- #
# General utility functions #
# ------------------------- #

#######################################
# Spit out a formatted error message to stderr with a timestamp
# Arguments:
#   $1 - error message
#######################################
err() {
  echo "[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $*" >&2
}

#######################################
# Spit out a formatted error message to stderr with a timestamp and quits
# Arguments:
#   $1 - error message
#######################################
fatal() {
  err "$1"
  exit 1
}

#######################################
# Ensure required CLI binaries are install, otherwise quit
# Arguments:
#   None
#######################################
ensure_requirements() {
  local required_cli_binaries=("yq" "curl" "envsubst")
  local missing=()

  for b in "${required_cli_binaries[@]}"; do
    if ! command -v "$b" &>/dev/null; then
      missing+=("$b")
    fi
  done

  if [[ "${#missing[@]}" -gt 0 ]]; then
    fatal "Missing required binaries: ${missing[@]}"
  fi
}

# ---- #
# Exec #
# ---- #

main() {
  source ./installers.sh

  if [[ "$?" != 0 ]]; then
    fatal "Failed to source installers.sh"
  fi

  while [[ "$1" =~ ^- && ! "$1" == "--" ]]; do
    case "$1" in
      -d | --dry-run ) DRY_RUN=1 ;;
    esac
    shift
  done
  if [[ "$1" == '--' ]]; then shift; fi # I don't need this, but it doesn't hurt

  local yaml_file="$1"
  if [[ ! -e "$yaml_file" ]]; then
    fatal "$yaml_file does not exist"
  elif [[ ! -f "$yaml_file" ]]; then
    fatal "$yaml_file is not a file"
  elif ! yaml_error="$(yq '.' "$yaml_file" 2>&1 >/dev/null)"; then
    fatal "$yaml_error"
  fi

  ensure_requirements

  if [[ $(yq '.flatpak' "$yaml_file") != "null" ]]; then
    install::flatpaks $yaml_file
  fi
  if [[ $(yq '.binary' "$yaml_file") != "null" ]]; then
    install::binaries $yaml_file
  fi
}

main "$@"
