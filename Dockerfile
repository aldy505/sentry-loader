FROM node:20.15-bookworm AS js-builder

WORKDIR /

ARG SENTRY_JS_VERSION=8.19.0

RUN git clone --depth 1 --branch ${SENTRY_JS_VERSION} https://github.com/getsentry/sentry-javascript.git && \
    cd sentry-javascript && \
    yarn install --frozen-lockfile --ignore-engines --ignore-scripts && \
    yarn workspace @sentry/types build && \
    yarn workspace @sentry/utils build && \
    yarn workspace @sentry/core build && \
    yarn workspace @sentry-internal/integration-shims build && \
    yarn workspace @sentry-internal/browser-utils build && \
    yarn workspace @sentry-internal/replay-worker build && \
    yarn workspace @sentry-internal/replay build && \
    yarn workspace @sentry-internal/replay-canvas build && \
    yarn workspace @sentry-internal/feedback build && \
    yarn workspace @sentry/browser build

FROM rust:1.79-bookworm AS server-builder

WORKDIR /build

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /var/sentry-loader/

RUN apt-get update && \
    apt-get install -y openssl libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

ENV JS_SDK_VERSION=${SENTRY_JS_VERSION}

COPY templates /var/sentry-loader/templates/

COPY --from=server-builder /build/target/release/sentry-loader /usr/bin/sentry-loader

COPY --from=js-builder /sentry-javascript/packages/browser/build/bundles /var/sentry-loader/bundles/

CMD ["/usr/bin/sentry-loader"]