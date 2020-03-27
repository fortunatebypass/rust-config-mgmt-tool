#!/bin/bash
cargo build
docker run --rm -it -v $(pwd):/app ubuntu:14.04 /app/tests/test.sh
