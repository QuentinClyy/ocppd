FROM alpine:3.22 AS certificates
RUN apk add --no-cache ca-certificates

FROM scratch
ARG TARGETARCH
COPY --from=certificates /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY dist/${TARGETARCH}/ocppd /ocppd
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
USER 65534:65534
EXPOSE 7600
ENTRYPOINT ["/ocppd", "--listen", "tcp:0.0.0.0:7600"]
