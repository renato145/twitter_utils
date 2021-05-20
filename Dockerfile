FROM rust:1.52.1-slim

RUN mkdir /app
WORKDIR /app
COPY twitter_stream/ .
RUN apt-get update
RUN apt-get install -y make pkg-config libssl-dev libzmq3-dev
RUN make all
RUN rm -rf target
RUN mv twitter_stream /usr/local/bin \
    && mv zmq_elasticsearch /usr/local/bin \
    && mv zmq_publisher /usr/local/bin

ENTRYPOINT [ "twitter_stream" ]
