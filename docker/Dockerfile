FROM rust:1.96-alpine3.23 AS base
RUN cargo install cargo-chef --version ^0.1

FROM base AS planner
WORKDIR /src
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /src
COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --target "$(uname -m)-unknown-linux-musl"
COPY . .
ARG CHECKMK_KUBERNETES_AGENT_GIT_SHA
RUN cargo build --release --target "$(uname -m)-unknown-linux-musl"

FROM alpine:3.23 AS node-scraper
ARG CMK_AGENT_SOURCE_TAG="v2.5.0p9"
ARG CHECKMK_AGENT_DIR="/opt/omd/versions/default/share/check_mk/agents"
ARG CHECKMK_AGENT_PLUGIN_DIR="${CHECKMK_AGENT_DIR}/plugins"
RUN wget https://raw.githubusercontent.com/Checkmk/checkmk/refs/tags/${CMK_AGENT_SOURCE_TAG}/agents/check_mk_agent.openwrt &&\
    wget https://raw.githubusercontent.com/Checkmk/checkmk/refs/tags/${CMK_AGENT_SOURCE_TAG}/agents/plugins/mk_inventory.linux &&\
    wget https://raw.githubusercontent.com/Checkmk/checkmk/refs/tags/${CMK_AGENT_SOURCE_TAG}/agents/plugins/lnx_container_host_if.linux &&\
    install -m 0755 check_mk_agent.openwrt mk_inventory.linux lnx_container_host_if.linux /usr/local/bin/ &&\
    rm -f check_mk_agent.openwrt mk_inventory.linux lnx_container_host_if.linux &&\
    # coreutils is *required* because busybox `timeout` creates zombies and we leak PIDs
    apk add coreutils=9.8-r1
COPY --from=builder /src/target/*/release/node-scraper /usr/local/bin/
COPY docker/check_mk_agent /usr/local/bin/check_mk_agent
RUN addgroup appgroup -g 10001 && adduser -HD appuser -G appgroup -u 10001
# A numeric USER lets Kubernetes verify runAsNonRoot without fixing the Pod UID.
USER 10001:10001
ENV PATH="/src/target/debug:/src/target/x86_64-unknown-linux-musl/debug:/src/target/aarch64-unknown-linux-musl/debug:${PATH}"
ENTRYPOINT ["node-scraper"]

FROM alpine:3.23 AS cluster-aggregator
COPY --from=builder /src/target/*/release/cluster-aggregator /usr/local/bin/
RUN addgroup appgroup -g 10001 && adduser -HD appuser -G appgroup -u 10001
USER 10001:10001
ENV PATH="/src/target/debug:/src/target/x86_64-unknown-linux-musl/debug:/src/target/aarch64-unknown-linux-musl/debug:${PATH}"
ENTRYPOINT ["cluster-aggregator"]

# dev variants don't build the rust binaries Docker, they expect them mounted
# in to /src in the respective pods.

FROM alpine:3.23 AS node-scraper-dev
ARG CMK_AGENT_SOURCE_TAG="v2.5.0p9"
ARG CHECKMK_AGENT_DIR="/opt/omd/versions/default/share/check_mk/agents"
ARG CHECKMK_AGENT_PLUGIN_DIR="${CHECKMK_AGENT_DIR}/plugins"
RUN wget https://raw.githubusercontent.com/Checkmk/checkmk/refs/tags/${CMK_AGENT_SOURCE_TAG}/agents/check_mk_agent.openwrt &&\
    wget https://raw.githubusercontent.com/Checkmk/checkmk/refs/tags/${CMK_AGENT_SOURCE_TAG}/agents/plugins/mk_inventory.linux &&\
    wget https://raw.githubusercontent.com/Checkmk/checkmk/refs/tags/${CMK_AGENT_SOURCE_TAG}/agents/plugins/lnx_container_host_if.linux &&\
    install -m 0755 check_mk_agent.openwrt mk_inventory.linux lnx_container_host_if.linux /usr/local/bin/ &&\
    rm -f check_mk_agent.openwrt mk_inventory.linux lnx_container_host_if.linux &&\
    # coreutils is *required* because busybox `timeout` creates zombies and we leak PIDs
    apk add coreutils=9.8-r1
COPY docker/check_mk_agent /usr/local/bin/check_mk_agent
RUN addgroup appgroup -g 10001 && adduser -HD appuser -G appgroup -u 10001
USER 10001:10001
ENV PATH="/src/target/debug:/src/target/x86_64-unknown-linux-musl/debug:/src/target/aarch64-unknown-linux-musl/debug:${PATH}"
ENTRYPOINT ["node-scraper"]

FROM alpine:3.23 AS cluster-aggregator-dev
RUN addgroup appgroup -g 10001 && adduser -HD appuser -G appgroup -u 10001
USER 10001:10001
ENV PATH="/src/target/debug:/src/target/x86_64-unknown-linux-musl/debug:/src/target/aarch64-unknown-linux-musl/debug:${PATH}"
ENTRYPOINT ["cluster-aggregator"]
