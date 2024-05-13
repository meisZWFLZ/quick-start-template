FROM mcr.microsoft.com/devcontainers/rust:bullseye

ARG TYPST_VERSION=v0.11.0
ARG NOTEBOOKiNATOR_VERSION=1.0.1

# Install Typst
RUN wget https://github.com/typst/typst/releases/download/${TYPST_VERSION}/typst-x86_64-unknown-linux-musl.tar.xz && \
    tar -xvf typst-x86_64-unknown-linux-musl.tar.xz && \
    mv typst-x86_64-unknown-linux-musl/typst /usr/local/bin/typst && \
    rm -rf ./typst-x86_64-unknown-linux-musl*

# Install typst-live
RUN cargo install typst-live
ENV PATH="/root/.cargo/bin:${PATH}"

# Install the notebookinator
# We need to change the user here because typst local packages are installed per user
USER vscode
WORKDIR /home/vscode/
RUN git clone --depth 1 --branch ${NOTEBOOKiNATOR_VERSION} https://github.com/The-Notebookinator/notebookinator.git && \
    ./notebookinator/scripts/package @local && \
   rm -rf notebookinator
