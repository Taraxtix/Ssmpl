.PHONY: debug release build_debug build_release link

debug: build_debug link

release: build_release link

build_debug:
	cargo build $(EXEC)

build_release:
	cargo build --release $(EXEC)

link:
	rm -f ./ssmpl && ln -s target/debug/ssmpl .
