files = twitter_stream explore_tweets
target_dir = target/release

.PHONY: all build move

all: build move

build:
	cargo build --release

move:
	for file in ${files}; do mv ${target_dir}/$$file .; done

clean:
	for file in ${files}; do rm $$file; done
