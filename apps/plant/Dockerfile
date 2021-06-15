FROM rustlang/rust:nightly

WORKDIR /usr/src/app

RUN rustup target add thumbv6m-none-eabi

COPY . .

CMD ["cargo", "build", "--release"]
