#!/bin/bash
set -eoux pipefail

count() {
    rg --ignore-case --count-matches --include-zero "\b$1\b" rfcs > $1.txt
}

count 'except'
count 'unless'
count 'should'
count 'unspecified'
count 'required'
count 'must'
count 'may'
