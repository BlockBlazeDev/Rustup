#!/bin/sh

INPATH=$(dirname "$0")
OUTPATH="${INPATH}/../../.github/workflows/ci.yaml"

gen_job () {
    grep -v "skip-$2" "$INPATH/$1-template.yaml" >> "$OUTPATH"
}

cat << EOF > "$OUTPATH"
# GitHub Actions workflow generated by ci/actions-templates/gen-workflows.sh
# Do not edit this file in .github/workflows

name: CI

on:
  pull_request:
    branches:
      - "*"
  push:
    branches:
      - master
      - stable
      - renovate/*
  schedule:
    - cron: "30 0 * * 1" # Every Monday at half past midnight UTC

jobs:
EOF

gen_job windows-builds pr
gen_job windows-builds master
gen_job windows-builds stable

gen_job linux-builds pr
gen_job linux-builds master
gen_job linux-builds stable

# The following targets only have a single job
gen_job macos-builds all
gen_job freebsd-builds all
gen_job centos-fmt-clippy all
gen_job all-features all
gen_job test-docs all
