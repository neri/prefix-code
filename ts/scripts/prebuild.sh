#! /bin/sh
echo "export const HASH = \"`git rev-parse --short HEAD`\";" > src/hash.ts
