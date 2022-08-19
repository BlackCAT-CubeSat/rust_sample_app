# Copyright (c) 2022 The Pennsylvania State University and the project contributors.
# SPDX-License-Identifier: Apache-2.0

cmake_minimum_required(VERSION 3.13.0)

project(CFE_RUST_SAMPLE_APP C)

add_cfe_app(rust_sample fsw/src/placebo.c)

cfe_rust_crate(rust_sample rust_sample)

target_link_options(rust_sample PUBLIC LINKER:--require-defined=SAMPLE_APP_Main)
