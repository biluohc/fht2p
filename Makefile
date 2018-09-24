br:
	echo "\033[35m  正式编译于： `TZ=UTC-8 date +"%Y-%m-%d %H:%M:%S"` \033[0m"

	pwd && ls -al && cargo build --release

b:
	echo "\033[35m  编译于： `TZ=UTC-8 date +"%Y-%m-%d %H:%M:%S"` \033[0m"

	pwd && ls -al && cargo build

d: 
	echo "\033[35m  文档编译于： `TZ=UTC-8 date +"%Y-%m-%d %H:%M:%S"` \033[0m" 

	cargo doc

f:
	echo "\033[35m  格式化于： `TZ=UTC-8 date +"%Y-%m-%d %H:%M:%S"` \033[0m"
	cargo fmt

a: 
	make br && echo -n && make b && echo -n && make d

c: 
	cargo clean

t: 
	cargo test -- --nocapture 

tr: 
	cargo test --release -- --nocapture 

