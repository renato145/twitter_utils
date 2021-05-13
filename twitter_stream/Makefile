fname = twitter_stream
target_dir = target/release

.PHONY: all build move

all: build clean move

build:
	cargo build --release

clean:
	rm ${fname}

move:
	mv ${target_dir}/${fname} .
