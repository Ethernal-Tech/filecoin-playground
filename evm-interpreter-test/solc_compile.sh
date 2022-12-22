#!/bin/bash

input_file=$1
output_dir=$2

# TODO: remove verbose
solcjs $input_file --bin -o $output_dir --verbose