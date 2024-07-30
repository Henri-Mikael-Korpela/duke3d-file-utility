FROM alpine:3.19

# Install Git
RUN apk add --no-cache git

# Install Rust and its associated tools
RUN apk add curl
RUN apk add gcc
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Configure the PATH so that Cargo is available
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /main

# Keep the container running
CMD ["tail", "-f", "/dev/null"]