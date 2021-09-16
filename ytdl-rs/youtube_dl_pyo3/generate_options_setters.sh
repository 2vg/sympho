#!/usr/bin/env bash
function read_file() {
    while read p; do
        echo "$p" | grep -v "//"
    done < struct
}

function get_names() {
    for line in $(read_file); do
        echo $line | grep -v Option | grep -v pub | sed 's/://g'
    done
}

function get_types() {
    for line in $(read_file); do
        echo $line | grep Option | grep -v pub | sed 's/,//g;s/Option<//g;s/>$//g;'
    done
}

names=($(get_names))
types=($(get_types))

for i in "${!names[@]}"; do
	echo "declare_options_setter!(${names[$i]}, ${types[$i]});";
done
