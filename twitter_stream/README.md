# Twitter Stream

Utility to stream data from twitter into a json line file. It also include subcommands to help creating and deleting stream rules.

## Examples
- explore_tweets: deserializes and prints last tweets.
- zmq_publisher: simple publisher using ZeroMQ.
- zmq_sub: simple subscriber using ZeroMQ.
- zmq_elasticsearch: worker that saves messages on Elastic Search (check `run_elastic_search.sh`).
- jsonl2es: dumps the entire content of a JSON Lines file to Elastic Search.
