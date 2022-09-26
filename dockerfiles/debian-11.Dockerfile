FROM debian:11
ENV PATH="/root/.cargo/bin:${PATH}"
RUN apt update -y && apt install -y gcc curl bzip2 lcov libssl-dev pkg-config
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
RUN cargo --version

