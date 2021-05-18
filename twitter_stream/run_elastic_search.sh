#!/bin/bash
docker run -d --rm --name elasticsearch \
    -v ${PWD}/elasticsearch_data:/usr/share/elasticsearch/data \
    -p 9200:9200 -p 9300:9300 \
    -e "discovery.type=single-node" \
    -e "ES_JAVA_OPTS=-Xms512m -Xmx512m" \
    elasticsearch:7.12.1
