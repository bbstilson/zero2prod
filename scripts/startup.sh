#! /usr/bin/bash

set -eu pipefail


echo "Launching redis..."
./scripts/init_redis.sh

echo "Launching postgres..."
./scripts/init_db.sh

echo "Ready!"
