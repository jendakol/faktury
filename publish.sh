#!/bin/bash

docker tag faktury:dev "jendakol/faktury:$1"
docker tag faktury:dev "jendakol/faktury:latest"
docker push "jendakol/faktury:$1"
docker push "jendakol/faktury:latest"
