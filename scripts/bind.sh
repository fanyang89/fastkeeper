#!/usr/bin/env bash

set -x

pushd zookeeper-client
cbindgen --output include/zookeeper.h --lang c .
popd
