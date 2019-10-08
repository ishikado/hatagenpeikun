FROM ubuntu:18.04
RUN apt-get update
RUN apt-get -y install curl git zsh gcc g++ pkg-config make
RUN apt-get -y install libssl-dev

SHELL ["/bin/zsh", "-c"]

RUN curl https://sh.rustup.rs > setup.sh
RUN sh setup.sh -y

ARG LOG_LEVEL
ARG SLACK_API_TOKEN

ENV LEVEL $LOG_LEVEL
ENV TOKEN $SLACK_API_TOKEN
ENV PATH $PATH:/root/.cargo/bin

# fix me!!
COPY . hatagenpeikun

RUN cd rust_test_bot &&  cargo install --path .
CMD ["/bin/zsh", "-c", "/root/.cargo/bin/hatagenpeikun $TOKEN -l $LEVEL -r $REDIS_URL"]
