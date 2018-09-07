FROM alpine:edge

COPY ./directemar_crawler /directemar_crawler

# comment out following 2 lines if not using SSL
#RUN apk update && apk add --no-cache ca-certificates && update-ca-certificates
#ENV SSL_CERT_DIR=/etc/ssl/certs

ENTRYPOINT [ "/directemar_crawler" ]
