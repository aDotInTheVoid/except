#!/bin/bash
set -eou pipefail

i="$1"

i_trimmed=$(echo "$1" | sed 's/^0*//')
url="https://www.rfc-editor.org/rfc/rfc$i_trimmed.txt"

# First char of $i
i0=${i:0:1}
# Second char of $i
i1=${i:1:1}
# Third char of $i
i2=${i:2:1}
# Fourth char of $i
i3=${i:3:1}

file="rfcs/$i0/$i1/$i2/$i3/$1.txt"

mkdir -p $(dirname $file)

curl $url -o $file
