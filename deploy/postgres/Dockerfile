# PostgreSQL 18 + pgvector 0.8.x (from base image) + Apache AGE 1.7.x.
#
# Built on top of pgvector's official image so we don't have to rebuild
# pgvector from source. AGE is added by cloning + `make install` against the
# PG18 branch.

FROM pgvector/pgvector:pg18

ARG AGE_REF=PG18/v1.7.0-rc0

USER root
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        flex \
        bison \
        git \
        libreadline-dev \
        postgresql-server-dev-18 \
        zlib1g-dev; \
    git clone --depth 1 --branch "${AGE_REF}" https://github.com/apache/age.git /tmp/age; \
    cd /tmp/age; \
    make install; \
    cd /; \
    rm -rf /tmp/age; \
    apt-get purge -y --auto-remove \
        build-essential \
        flex \
        bison \
        git \
        libreadline-dev \
        postgresql-server-dev-18 \
        zlib1g-dev; \
    rm -rf /var/lib/apt/lists/*

# Pre-flight init: enable both extensions on the bootstrap database.
COPY init.sql /docker-entrypoint-initdb.d/00-plutus-init.sql

USER postgres
