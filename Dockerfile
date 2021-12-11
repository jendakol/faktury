FROM debian:bullseye-slim

WORKDIR /tmp/faktury

COPY . .

RUN apt-get update \
 && apt-get install -y curl file sudo gcc libsodium-dev clang-7.0 pkg-config make libssl-dev liblzma-dev default-libmysqlclient-dev nodejs \
 && curl -sL https://deb.nodesource.com/setup_16.x | bash - \
 && curl -L https://npmjs.org/install.sh | sudo sh \
 && curl https://sh.rustup.rs -sSf | sh -s -- -y \
 && export PATH="$HOME/.cargo/bin:$PATH" \
 && cargo build --release \
 && cd front && npm i && npm run build && cd .. \
 && apt-get remove -y curl file gcc pkg-config make clang-7.0 nodejs \
 && apt-get autoremove -y \
 && mkdir /faktury \
 && mv target/release/faktury /faktury/bin \
 && mv front/dist /faktury/static \
 && mv fonts /faktury \
 && mv imgs /faktury \
 && mv config /faktury \
 && rm -rf /tmp/faktury /root/.cargo /root/.rustup /root/.npm /usr/lib/node_modules /var/lib/apt /var/lib/dpkg /var/cache \
 && rm -rf /usr/lib/gcc /usr/lib/llvm-11 /usr/lib/python* /usr/share/cargo /usr/lib/python* /usr/lib/x86_64-linux-gnu/perl* /usr/include/llvm* \
 && rm -rf /usr/lib/x86_64-linux-gnu/libLLVM-11.so.1 /usr/lib/x86_64-linux-gnu/libclang-11.so.1 /usr/lib/x86_64-linux-gnu/libz3.so.4 \
 && rm -rf /usr/bin/python* /usr/bin/perl* /usr/include/c++

ENV RUST_LOG="debug,mio=info"

WORKDIR /faktury

ENTRYPOINT ["/faktury/bin"]
