#!/bin/bash
docker run -d --rm --name elasticsearch -v ${PWD}/elasticsearch_data:/usr/share/elasticsearch/data -p 9200:9200 -p 9300:9300 -e "discovery.type=single-node" elasticsearch:7.12.1
