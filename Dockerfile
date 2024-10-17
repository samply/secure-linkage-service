FROM alpine AS chmodder
ARG FEATURE
ARG TARGETARCH
ARG COMPONENT
COPY /artifacts/binaries-$TARGETARCH$FEATURE/$COMPONENT /app/$COMPONENT
RUN chmod +x /app/*

FROM gcr.io/distroless/cc-debian12
ARG COMPONENT
COPY --from=chmodder /app/$COMPONENT /usr/local/bin/securelinkageservice
ENTRYPOINT [ "/usr/local/bin/securelinkageservice" ]
