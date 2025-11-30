#!/bin/bash

i=0
while true; do
  echo "Log entry #$i - $(date)"
  i=$((i+1))
  sleep 1
done
