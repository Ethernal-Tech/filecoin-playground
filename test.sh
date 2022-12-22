#!/bin/bash

search_dir_applications="contracts/applications"
search_dir_hacks="contracts/hacks"
search_dir_simple_examples="contracts/simple_examples"

for entry in "$search_dir_applications"/*.sol
do
    echo "File: $entry"
    bash solc_compile.sh $entry "contracts/out"
done

for entry in "$search_dir_hacks"/*.sol
do
    echo "File: $entry"
    bash solc_compile.sh $entry "contracts/out"
done

for entry in "$search_dir_simple_examples"/*.sol
do
    echo "File: $entry"
    bash solc_compile.sh $entry "contracts/out"
done