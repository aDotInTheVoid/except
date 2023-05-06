#!/bin/bash


i=$(echo "$1" | sed 's/^0*//')
url="https://www.rfc-editor.org/rfc/rfc$i.txt"

curl "$url" -o rfcs/$1.txt
