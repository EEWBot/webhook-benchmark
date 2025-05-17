FROM rust:1.87.0-bookworm AS build-env
LABEL maintainer="yanorei32"

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

WORKDIR /usr/src
COPY . /usr/src/webhook-benchmark/
WORKDIR /usr/src/webhook-benchmark
RUN cargo build --release && cargo install cargo-license && cargo license \
	--authors \
	--do-not-bundle \
	--avoid-dev-deps \
	--avoid-build-deps \
	--filter-platform "$(rustc -vV | sed -n 's|host: ||p')" \
	> CREDITS

FROM debian:bookworm-slim

RUN apt-get update; \
	apt-get install -y --no-install-recommends \
		libssl3 ca-certificates; \
	apt-get clean;

WORKDIR /

COPY --chown=root:root --from=build-env \
	/usr/src/webhook-benchmark/CREDITS \
	/usr/src/webhook-benchmark/LICENSE \
	/usr/share/licenses/webhook-benchmark/

COPY --chown=root:root --from=build-env \
	/usr/src/webhook-benchmark/target/release/webhook-benchmark \
	/usr/bin/webhook-benchmark

CMD ["/usr/bin/webhook-benchmark"]
