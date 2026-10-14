# The Samply Rust workflow builds the binaries and stores them under artifacts/.
# This Dockerfile copies the workflow-built binary into a small runtime image.

FROM alpine AS chmodder
ARG FEATURE
ARG TARGETARCH
ARG COMPONENT
COPY /artifacts/binaries-$TARGETARCH$FEATURE/$COMPONENT /app/$COMPONENT
RUN chmod +x /app/*

FROM gcr.io/distroless/cc-debian12
ARG COMPONENT
ENV HOST=0.0.0.0
ENV PARTIES_CONFIG_FILE=/etc/secure-linkage-service/linkage-parties.toml
COPY linkage-parties.toml /etc/secure-linkage-service/linkage-parties.toml
COPY --from=chmodder /app/$COMPONENT /usr/local/bin/secure-linkage-service
ENTRYPOINT ["/usr/local/bin/secure-linkage-service"]
