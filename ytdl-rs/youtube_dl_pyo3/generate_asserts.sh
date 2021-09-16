#!/usr/bin/env bash

function read_file() {
    while read p; do
        echo "$p" | grep -v "//"
    done < struct
}

function get_names() {
    for line in $(read_file); do
        echo $line;
    done | grep -v Option | grep -v pub | grep -v // | sed 's/://g'
}

for name in $(get_names); do
    echo "assert!(pydict.get_item(\"$name\").is_none());";
done
