#!/usr/bin/env bash

version=$(cargo metadata --format-version 1 | jq -r --arg pkg_name "uic-dosipas-lib" '.packages[] | select(.name==$pkg_name) .version')
git tag -f $version
git push --tags --force