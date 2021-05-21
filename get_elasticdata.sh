#!/bin/bash
mkdir -p data/raw
cd data/raw
FN="${1:-data.jsonl}"
echo "Saving data on: ${PWD}/data/raw/${FN}"
elasticdump --input=http://localhost:9200/tweets --output=data_tmp.jsonl --type=data --limit=5000
jq -c '._source' data_tmp.jsonl > $FN
rm data_tmp.jsonl
echo "Data saved on: ${PWD}/data/raw/${FN}"
