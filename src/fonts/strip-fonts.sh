#!/bin/sh
find . -name '*.ttf' ! -name '*stripped.ttf' | while read f
do
    OUT="$(echo $f | cut -d'.' -f1-2)-stripped.ttf"
    EXE="../../target/release/excavation-site-mercury"
    pyftsubset "$f" --output-file="$OUT" --text-file=<(strings "$EXE")
done
