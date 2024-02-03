#!/bin/bash

cargo leptos build --release -vv &&\
    mkdir -p distribute &&\
    cp -r target/site distribute/site &&\
    cp target/release/ferris-the-waiter distribute/ &&\
    cp Cargo.toml distribute/ &&\
    tar -czvf ferris-the-waiter.tar.gz --directory=distribute . &&\
    rm -rf distribute
