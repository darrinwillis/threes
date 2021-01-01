
.PHONY: streamlit

streamlit: threes_engine
	pipenv run streamlit run src/analysis_app.py

threes_engine:
	cargo build

train:
	cargo run --release -- train

train_d:
	cargo run -- train

test:
	cargo test

play:
	cargo run --release -- interactive

help:
	cat Makefile
	cargo run -- --help
