#!/bin/bash
find .. -name \*.rs -or -name \*.html | grep -v output | entr -cr ./build-and-serve.sh
