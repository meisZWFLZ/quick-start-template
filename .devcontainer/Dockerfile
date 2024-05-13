FROM mcr.microsoft.com/devcontainers/base:bullseye

ARG TYPST_VERSION=v0.11.0
ARG NOTEBOOKiNATOR_VERSION=1.0.1

RUN wget https://github.com/typst/typst/releases/download/${TYPST_VERSION}/typst-x86_64-unknown-linux-musl.tar.xz && \
    tar -xvf typst-x86_64-unknown-linux-musl.tar.xz && \
    mv typst-x86_64-unknown-linux-musl/typst /usr/local/bin/typst && \
    rm -rf ./typst-x86_64-unknown-linux-musl*

USER vscode
WORKDIR /home/vscode/
RUN git clone --depth 1 --branch ${NOTEBOOKiNATOR_VERSION} https://github.com/The-Notebookinator/notebookinator.git && \
    ./notebookinator/scripts/package @local && \
   rm -rf notebookinator