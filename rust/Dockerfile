ARG BASE_IMAGE=common-tools-base
ARG EXECUTION_IMAGE=debian:bookworm-slim
ARG BINARY_PATH="/bin/bash"
ARG EXPOSED_PORT=

FROM ${BASE_IMAGE} AS base

COPY . .

# Build the project
RUN make --debug=j local

RUN make --debug=j list_outputs

FROM ${EXECUTION_IMAGE} AS execution-image

ARG FILES="target/*"

COPY --from=base ${FILES} .

EXPOSE ${EXPOSED_PORT}
ENTRYPOINT [$BINARY_PATH]
