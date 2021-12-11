#!/bin/bash

rm -rf /tmp/faktury-build
mkdir /tmp/faktury-build

cd /home/jenda/dev/faktury
cp -r config /tmp/faktury-build
cp -r fonts /tmp/faktury-build
cp -r imgs /tmp/faktury-build
cp -r migrations /tmp/faktury-build
cp -r src /tmp/faktury-build
cp -r Cargo* /tmp/faktury-build
cp -r diesel.toml /tmp/faktury-build
cp -r Dockerfile /tmp/faktury-build

mkdir /tmp/faktury-build/front
cd front
cp -r public /tmp/faktury-build/front
cp -r sass /tmp/faktury-build/front
cp -r src /tmp/faktury-build/front
cp -r *.js /tmp/faktury-build/front
cp -r package*.json /tmp/faktury-build/front

cd /tmp/faktury-build
docker build -t faktury:dev .

