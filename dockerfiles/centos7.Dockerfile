FROM centos:7
ENV PATH="/root/.cargo/bin:${PATH}"
RUN yum -y install curl gcc bzip2 openssl openssl-devel
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
RUN cargo --version

