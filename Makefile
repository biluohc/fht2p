br:
	echo "\033[35m  Release compiled at: `TZ=UTC-8 date +"%Y-%m-%d %H:%M:%S"` \033[0m"
	pwd && ls -al && cargo build --release

b:
	echo "\033[35m  Compiled at: `TZ=UTC-8 date +"%Y-%m-%d %H:%M:%S"` \033[0m"
	pwd && ls -al && cargo build

d: 
	echo "\033[35m  Document compiled at: `TZ=UTC-8 date +"%Y-%m-%d %H:%M:%S"` \033[0m" 
	cargo doc

f:
	echo "\033[35m  Code format at: `TZ=UTC-8 date +"%Y-%m-%d %H:%M:%S"` \033[0m"
	cargo fmt

a: 
	make br && echo -n && make b && echo -n && make d

c: 
	rm -frv tests/upload/*.rs
	cargo check

t: 
	cargo test -- --nocapture 

tr: 
	cargo test --release -- --nocapture 

clippy:
	cargo clippy

# musl-gcc & musl-ldd
# apt install -y musl-tools && dpkg -L musl-tools
# env RUSTUP_DIST_SERVER=https://mirrors.sjtug.sjtu.edu.cn/rust-static rustup target add x86_64-unknown-linux-musl
musl:
	rustup target add x86_64-unknown-linux-musl && cargo build --release --target x86_64-unknown-linux-musl

# docker run -v $PWD:/volume  -v $CACHE:/volume/target -v $CACHE:/root/.cargo/registry  --rm -t clux/muslrust cargo build --release 
musld:
	docker pull clux/muslrust && docker run -v `pwd`:/volume --rm -t clux/muslrust cargo build --release
