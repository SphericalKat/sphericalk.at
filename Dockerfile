FROM rustlang/rust:nightly AS builder

RUN rustup target add x86_64-unknown-linux-musl --toolchain=nightly

WORKDIR /app

RUN curl -sL https://deb.nodesource.com/setup_18.x | bash -

RUN apt -y install nodejs

RUN npm i -g yarn

COPY Cargo.toml Cargo.lock package.json yarn.lock ./

RUN yarn install

COPY . .

RUN yarn build

RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch

COPY --from=builder /usr/local/cargo/bin/lithograph .

USER 1000

CMD ["./lithograph"]
