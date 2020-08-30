FROM debian:buster-slim

WORKDIR /tmp/faktury

COPY . .

RUN apt-get update \
 && apt-get install -y curl file sudo gcc libsodium-dev clang-6.0 pkg-config make libssl-dev liblzma-dev default-libmysqlclient-dev nodejs \
 && curl -sL https://deb.nodesource.com/setup_14.x | bash - \
 && curl -L https://npmjs.org/install.sh | sudo sh \
 && curl https://sh.rustup.rs -sSf | sh -s -- -y \
 && export PATH="$HOME/.cargo/bin:$PATH" \
 && cargo build --release \
 && cd front && npm i && npm run build && cd .. \
 && apt-get remove -y curl file gcc pkg-config make clang-6.0 nodejs \
 && apt-get autoremove -y \
 && mkdir /faktury \
 && mv target/release/faktury /faktury/bin \
 && mv front/dist /faktury/static \
 && mv fonts /faktury \
 && mv imgs /faktury \
 && mv config /faktury \
 && rm -rf /tmp/faktury /root/.cargo /root/.rustup /root/.npm /usr/lib/node_modules /var/lib/apt /var/lib/dpkg /var/cache

ENV RUST_LOG="debug,mio=info"

WORKDIR /faktury

ENTRYPOINT ["/faktury/bin"]
