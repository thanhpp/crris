#!/usr/bin/bash

diesel setup
diesel migration run
cargo run