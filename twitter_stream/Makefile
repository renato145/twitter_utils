fname = twitter_stream
examples = zmq_pub
target_dir = target/release

.PHONY: all build move

all: build clean move

build:
	cargo build --release
	for fn in ${examples}; do cargo build --example $$fn --release; done

clean:
	-rm ${fname}
	-rm ${examples}

move:
	mv ${target_dir}/${fname} .
	for fn in ${examples}; do mv ${target_dir}/examples/$$fn .; done
