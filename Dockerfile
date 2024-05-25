FROM node:20.12-bookworm AS js-builder

WORKDIR /

ARG SENTRY_JS_VERSION=8.4.0

RUN apt-get update && apt-get install -y git && \
    git clone --depth 1 --branch ${SENTRY_JS_VERSION} https://github.com/getsentry/sentry-javascript.git && \
    cd sentry-javascript && \
    yarn install --frozen-lockfile --ignore-engines --ignore-scripts && \
    yarn run build

FROM rust:1.78-bookworm AS server-builder

WORKDIR /build

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /var/sentry-loader/

ENV JS_SDK_VERSION=${SENTRY_JS_VERSION}

COPY templates /var/sentry-loader/templates/

COPY --from=js-builder /sentry-javascript/packages/browser/build/bundles/ /var/sentry-loader/bundles/

COPY --from=server-builder /build/target/release/sentry-loader /usr/bin/sentry-loader

CMD ["/usr/bin/sentry-loader"]