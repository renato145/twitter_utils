# Twitter utils

Requirements:
1. Have a Twitter developer account to access the v2 of their API, check: https://developer.twitter.com/en/docs/tutorials/stream-tweets-in-real-time.
2. Get the bearer_token from twitter developer portal and put it on an `.env` file on the repo root, eg:
```
BEARER_TOKEN=XXXXXXXXXXX
```
3. Create stream rules:
   - First compile the docker image: `docker build -t twitter_stream .`
   - Then you can run some utilities for managing your stream rules:
     - list rules: `docker run --rm twitter_stream --bearer-token XXXX list-rules`
     - create rule: `docker run --rm twitter_stream --bearer-token XXXX create-rule "your_rule_goes_here" -t some_tag`
     - delete rule: `docker run --rm -it twitter_stream --bearer-token XXXX delete-rule`
   - Rule example: "python (machine OR deep) learning -is:retweet lang:en". This rule will stream any tweet with the words "python", "machine or deep" and "learning" that are not retweets and are in english. To add this rule under a tag like "data-science" run: `docker run --rm twitter_stream --bearer-token XXXX create-rule "python (machine OR deep) learning -is:retweet lang:en" -t data-science`
   - For more info on how to create rules check: https://developer.twitter.com/en/docs/twitter-api/tweets/filtered-stream/integrate/build-a-rule.


Once all desired rules are setup is time to stream. The easiest way to have everything installed and running is with docker-compose installed.

But, before running docker-compose create a folder called `elasticsearch_data` as otherwise things will fail.

By running the docker-compose file 4 services will start:
- elasticsearch: An instance of Elastic Search
- zmq_publisher: Obtains data from twitter stream and publishes messages via ZeroMQ (source code in `twitter_stream/examples/zmq_publisher.rs`).
- zmq_elasticsearch: Receives messages from the zmq_publisher and sends them to the elasticsearch instance (source code in `twitter_stream/examples/zmq_elasticsearch.rs`).
- kibana: An instance of Kibana for easy data exploration.

To see the state of the stream run: `docker logs twitter_utils_zmq_elasticsearch_1 -f`
To access Kibana go to http://localhost:5601 (the first time may take a couple of minutes to start), then you need to need to add the Elastic Search index in Kibana:

1. Go to "Discover" link in the side bar:

![Step1](imgs/1_discover.png)

2. Click on "Create index pattern":

![Step2](imgs/2_create-index.png)

3. Type the index patter name: "tweets*":

![Step3](imgs/3_define_index.png)

4. Select the time field and click on "Create index pattern":

![Step4](imgs/4_select-time-field.png)

After this you should be able to use Kibana tools to explore the data (check "Visualize Library" on the side bar).

> If you don't want to use docker, you will need rust and cargo installed to compile the binaries in `twitter_stream` folder by running `make all`.