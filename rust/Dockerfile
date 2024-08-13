ARG BASE_IMAGE=common-tools-base
ARG EXECUTION_IMAGE=debian:latest
ARG BINARY_PATH="/bin/bash"
ARG EXPOSED_PORT=

FROM ${BASE_IMAGE} as common-tools-monolith

COPY . .

# Build the project
RUN make build_typescript
RUN make build_rust

RUN make list_outputs

FROM ${EXECUTION_IMAGE} as runtime

ARG FILES="target/*"

COPY --from=common-tools-monolith ${FILES} .

EXPOSE ${EXPOSED_PORT}
ENTRYPOINT [$BINARY_PATH]
