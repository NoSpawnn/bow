#!/usr/bin/env bash

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

# ---- #
# Exec #
# ---- #

while [[ "$1" =~ ^- && ! "$1" == "--" ]]; do
  case "$1" in
    -d | --dry-run ) DRY_RUN=1 ;;
  esac
  shift
done
if [[ "$1" == '--' ]]; then shift; fi

main() {
  local yaml_file="$1"

  if [[ ! -e "$yaml_file" ]]; then
    err "$yaml_file does not exist"
    exit 1
  elif [[ ! -f "$yaml_file" ]]; then
    err "$yaml_file is not a file"
    exit 1
  elif ! yq eval "$yaml_file" &>/dev/null; then
    # there's gotta be a nicer way to do this? where's my if let...
    err "$(yq eval \"$yaml_file\")"
    exit 1
  fi

  install_flatpaks $yaml_file
}

main "$@"
