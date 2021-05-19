#!/bin/bash
docker run -d --rm --name kibana \
    --link elasticsearch:elasticsearch \
    -p 5601:5601 \
    kibana:7.12.1
