#!/usr/bin/env bash

if [ ! -d "build" ]; then
    mkdir build
fi

cd build
echo "*" >> .gitignore
cmake ..
cmake --build .
