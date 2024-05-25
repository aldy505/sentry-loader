FROM node:20.12-bookworm AS js-builder

WORKDIR /

ARG SENTRY_JS_VERSION=8.4.0

RUN git clone --depth 1 --branch ${SENTRY_JS_VERSION} https://github.com/getsentry/sentry-javascript.git && \
    cd sentry-javascript && \
    yarn install --frozen-lockfile --ignore-engines --ignore-scripts && \
    yarn workspace @sentry/types build && \
    yarn workspace @sentry/utils build && \
    yarn workspace @sentry/core build && \
    yarn workspace @sentry-internal/browser-utils build && \
    yarn workspace @sentry-internal/replay-worker build && \
    yarn workspace @sentry-internal/replay build && \
    yarn workspace @sentry-internal/replay-canvas build && \
    yarn workspace @sentry-internal/feedback build && \
    yarn workspace @sentry/browser build:bundle

FROM rust:1.78-bookworm AS server-builder

WORKDIR /build

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /var/sentry-loader/

ENV JS_SDK_VERSION=${SENTRY_JS_VERSION}

COPY templates /var/sentry-loader/

COPY --from=server-builder /build/target/release/sentry-loader /usr/bin/sentry-loader

COPY --from=js-builder /sentry-javascript/packages/browser/build/bundles /var/sentry-loader/

CMD ["/usr/bin/sentry-loader"]