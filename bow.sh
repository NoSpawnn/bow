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
  local required_cli_binaries=("curl" "envsubst")
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

# ---------------------- #
# Installation functions #
# ---------------------- #

#######################################
# Install flatpaks based on IDs and scopes in a YAML file
# Arguments:
#   $1 - yaml file relative to this script to read from
#######################################
install_flatpaks() {
    local default_scope=$(yq '.flatpak.default_scope' < $1)

    if [[ ! "$default_scope" =~ ^(user|system)$ ]]; then
      echo "Invalid default scope '$default_scope', must be one of: user, system"
      exit 1
    fi

    local flatpaks_to_install_user=()
    local flatpaks_to_install_system=()

    function add_to_relevant_list() {
        if [[ "$2" == "user" ]]; then
          flatpaks_to_install_user+=("$1")
        elif [[ "$2" == "system" ]]; then
          flatpaks_to_install_system+=("$1")
        else
          echo "Invalid scope '$2' for $1, must be one of: user, system"
          exit 1
        fi
    }

    # map entries (specify an ID and scope key)
    local yqexpr_object_flatpaks='
      .flatpak.present.[] |
      select(tag == "!!map") |
      [ .id, .scope ] |
      join("|")
    '
    while IFS='|' read -r fp_id fp_scope; do
      if [[ -z "$fp_id" && ! -z "$fp_scope" ]]; then
        echo "Defining only a 'scope' is invalid"
        exit 1
      fi
      add_to_relevant_list "$fp_id" "$fp_scope"
    done < <(yq eval "$yqexpr_object_flatpaks" < "$1")

    # str entries
    local yqexpr_simple_flatpaks='
      .flatpak.present.[] |
      select(tag == "!!str")
    '
    while read -r fp_id; do
      local fp_scope="$default_scope"
      add_to_relevant_list "$fp_id" "$fp_scope"
    done < <(yq eval "$yqexpr_simple_flatpaks" < "$1")

    if [[ "$DRY_RUN" == 1 ]]; then
      echo "Would install ${#flatpaks_to_install_system[@]} system flatpaks"
      printf '    %s\n' ${flatpaks_to_install_system[@]}
      echo "Would install ${#flatpaks_to_install_user[@]} user flatpaks"
      printf '    %s\n' ${flatpaks_to_install_user[@]}
    else
      for fp in "${flatpaks_to_install_system[@]}"; do
        flatpak install --system "$fp"
      done
      for fp in "${flatpaks_to_install_user[@]}"; do
        flatpak install --user "$fp"
      done
    fi
}

#######################################
# Install CLI binaries based on information in a YAML file
# Arguments:
#   $1 - yaml file relative to this script to read from
#######################################
install_binaries() {
  local tmp="$(mktemp -d)"
  local install_dir="$HOME/.local/bin"
  local -A bins_to_install

  local yqexpr_object_binaries='
    .binary.[] |
    [ .name, .url, .version ] |
    join("|")
  '
  while IFS='|' read -r bin_name bin_url bin_version; do
    local downloaded_file="$tmp/$bin_name"

    if [[ "$bin_url" == *'$VERSION'* ]]; then
      if [[ -z "$bin_version" ]]; then
        fatal "\$VERSION is used in URL for $bin_name but version has not been specified"
      else
        bin_url="$(VERSION="$bin_version" envsubst '$VERSION' < <(echo "$bin_url"))"
      fi
    fi

    bins_to_install["$bin_name"]="$bin_url"
  done < <(yq eval "$yqexpr_object_binaries" < "$1")

  if [[ "$DRY_RUN" == 1 ]]; then
    echo "Would install ${#bins_to_install[@]} binaries"
    for key in "${!bins_to_install[@]}"; do
      value="${bins_to_install[$key]}"
      printf '    %s from %s\n' "$key" "$value"
    done
  else
    curl -sL "$bin_url" > "$downloaded_file"
    cp "$downloaded_file" .
  fi
}

# ---- #
# Exec #
# ---- #

main() {
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
  elif ! yaml_error="$(yq eval '.' "$yaml_file" 2>&1 >/dev/null)"; then
    fatal "$yaml_error"
  fi

  ensure_requirements
  # install_flatpaks $yaml_file
  install_binaries $yaml_file
}

main "$@"
