#!/usr/bin/env bash

version=$(cargo metadata --format-version 1 | jq -r --arg pkg_name "uic-dosipas-lib" '.packages[] | select(.name==$pkg_name) .version')
sed -i "" -E "s/(version = \")[^\"]+(\")/\1$version\2/g" android/build.gradle.kts
sed -i "" -E "s/(let releaseTag = \")[^\"]+(\")/\1$version\2/g" Package.swift
git add android/build.gradle.kts Package.swift
git commit -S -m "Set release version"
git push
git tag -f $version
git push --tags --force