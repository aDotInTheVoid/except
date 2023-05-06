#!/bin/bash
set -eou pipefail


if [ "$1" = "__secret" ] 
then
    echo "$2 $(./get_name.sh $2)" 
else
    cat $1.txt | sd 'rfcs/\d{2}/(rfc\d+\.txt):(\d+)' '$2 $1' | sort -n | xargs -I % ./most.sh __secret %
fi