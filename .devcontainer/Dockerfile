# Install fonts using fnt
FROM alpine:latest as fonts-fnt
# # Get git to clone repos and install fnt's dependencies: chafa
RUN apk add --no-cache git make curl bash

# fake otfinfo and chafa as they are only required by fnt for a few commands but are always checked
COPY <<EOF /usr/bin/otfinfo
#!/bin/sh
echo "otfinfo doesn't actually exist on this system and was faked in order to force necessary fnt features to work"
exit 10
EOF
COPY <<EOF /usr/bin/chafa
#!/bin/sh
echo "chafa doesn't actually exist on this system and was faked in order to force necessary fnt features to work"
exit 11
EOF
RUN chmod a+x /usr/bin/otfinfo /usr/bin/chafa

# Download fnt
RUN git clone https://github.com/alexmyczko/fnt.git ./fnt
# Install and perform simple test
RUN cd ./fnt && make install && fnt
# Update fnt's known fonts
RUN fnt update

# Install fonts
ARG FNT_FONTS
RUN echo "$FNT_FONTS" | tr ' ' '\n' | xargs -rP 4 -I {} fnt install {}
RUN mv /usr/local/share/fonts /fonts

# Install virgil font (excalidraw)
FROM alpine:latest as fonts-virgil
RUN apk add --no-cache woff2
# Download compressed font file
RUN wget https://virgil.excalidraw.com/Virgil.woff2 -O ./Virgil.woff2
# Decompress to Virgil.ttf
RUN woff2_decompress Virgil.woff2
RUN rm Virgil.woff2

# Create final product
FROM mcr.microsoft.com/devcontainers/rust:bullseye as final

# Install typst-live
RUN cargo install typst-live
ENV PATH="/root/.cargo/bin:${PATH}"

# We need to change the user here because typst local packages are installed per user
USER vscode
WORKDIR /home/vscode/

# ARG TYPST_VERSION=v0.11.0
ARG NOTEBOOKiNATOR_VERSION=patch-1
# Install the notebookinator
RUN git clone --depth 1 --branch ${NOTEBOOKiNATOR_VERSION} https://github.com/meisZWFLZ/notebookinator.git && \
    ./notebookinator/scripts/package @local && \
    rm -rf notebookinator

# install fonts
COPY --from=fonts-fnt /fonts/* /usr/local/share/fonts/
COPY --from=fonts-virgil /Virgil.ttf /usr/local/share/fonts/
