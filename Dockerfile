FROM ubuntu:22.04
COPY ./target/release/my-http-server-sniffer ./target/release/my-http-server-sniffer
ENTRYPOINT ["./target/release/my-http-server-sniffer"]