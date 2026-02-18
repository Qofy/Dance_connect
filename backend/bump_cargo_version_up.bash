#!/usr/bin/env bash

function bump_version_up() {


  # Simple one-liner to increment patch version
  # sed -i 's/^version = "\([0-9]*\)\.\([0-9]*\)\.\([0-9]*\)"/version = "\1.\2.$(((\3 + 1)))"/e' Cargo.toml

  # Or a more robust script:

  #!/bin/bash
  # increment-version.sh

  local TOML_FILE=""
  TOML_FILE="${1:-./Cargo.toml}"

  if [ ! -f "$TOML_FILE" ]; then
    echo "Error: $TOML_FILE not found"
    exit 1
  fi

  # Extract current version
  local CURRENT=""
  CURRENT=$(grep '^version = ' "$TOML_FILE" | head -1 | sed 's/version =  "\(.*\)"/\1/')
  echo "✓ Version : $CURRENT"
  local CLEAR_CURRENT=""
  CLEAR_CURRENT=$( cut -d= -f2- <<< "$CURRENT" | sed 's/\"//g'  | sed "s/\'//g" | xargs)
  echo "✓ Version : $CLEAR_CURRENT"
  # Split into parts
  local -i MAJOR=0
  MAJOR=$( cut -d\. -f1 <<< "$CLEAR_CURRENT" | sed 's/\"//g' | sed "s/\'//g" | xargs)
  local -i MINOR=0
  MINOR=$( cut -d\. -f2 <<< "$CLEAR_CURRENT" | sed 's/\"//g' | sed "s/\'//g" | xargs)
  local -i PATCH=0
  PATCH=$( cut -d\. -f3 <<< "$CLEAR_CURRENT" | sed 's/\"//g' | sed "s/\'//g" | xargs)

  echo "✓ Version : $MAJOR . $MINOR . $PATCH"
  (( PATCH ++ ))
  echo "✓ Version : $MAJOR . $MINOR . $PATCH"
  # Increment patch
  local -i NEW_PATCH=$PATCH
  local NEW_VERSION=""
  NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"

  echo "✓     New : $MAJOR . $MINOR . $NEW_PATCH"
  echo "✓     New : $NEW_VERSION"

  grep -A 1  -B 2 -E '^version =' "$TOML_FILE"
  # Update Cargo.toml
  # sed -i "s/^version = \"$CURRENT\"/version = \"$NEW_VERSION\"/" "$TOML_FILE"
  ersetzeindatei "$TOML_FILE"  "$CLEAR_CURRENT" "$NEW_VERSION" -sift 2>&1
  wait
  grep -A 1  -B 2 -E '^version =' "$TOML_FILE"
  echo "DIFF ----------"
  git diff "$TOML_FILE" 2>&1  | grep "$NEW_VERSION"
  echo "-----------DIFF"
  local -i count_changes=0
  count_changes=$(git diff "$TOML_FILE" 2>&1  | wc -l)

  if [ ${count_changes} -gt 12 ] ; then
  {
    echo "ERROR: Update Bump version failed for file: $TOML_FILE
    Expected 1 line change I see  (${count_changes} ) more .. Reverting "
    git checkout "$TOML_FILE"
  }
  else
  {
    echo "✓ Version: $CURRENT → $NEW_VERSION"
  }
  fi




  # Or if you prefer using awk:

  # awk -F. '/^version = / {
  #   match($0, /([0-9]+\.[0-9]+\.)([0-9]+)/, arr)
  #   newver = arr[1] (arr[2] + 1)
  #   gsub(/([0-9]+\.[0-9]+\.[0-9]+)/, newver)
  #   }
  # 1' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml

  # Quickest approach - use this one-liner:

  # cd /repo/work/veloessen/projects/20250817_quote_maker/20250823_mitote_v026_bike_connect_backend_bas
  # e44/v023/backend && \
  # CURRENT=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/') &&
  # \
  # NEW=$(echo "$CURRENT" | awk -F. '{print $1"."$2"."($3+1)}') && \
  # sed -i "s/version = \"$CURRENT\"/version = \"$NEW\"/" Cargo.toml && \



} # end bump_version_up

bump_version_up "${1-}"



#
#
