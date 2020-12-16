#!/bin/bash
#
# This builds a release version, compresses, and hashes it for the purpose
# of releasing on homebrew via kevlarr/homebrew-jrny

cargo build --release
cd target/release
tar -czf jrny-mac.tar.gz jrny
echo $( shasum -a 256 jrny-mac.tar.gz )
