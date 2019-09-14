FROM ubuntu:18.04
RUN apt-get update
RUN apt-get -y install curl git zsh gcc g++ pkg-config make
RUN apt-get -y install libssl-dev

SHELL ["/bin/zsh", "-c"]

RUN curl https://sh.rustup.rs > setup.sh
RUN sh setup.sh -y

ENV PATH $PATH:$HOME/.cargo/bin
RUN $HOME/.cargo/bin/cargo

#RUN --mount=type=secret,id=ssh,target=/root/.ssh/id_rsa git clone git@bitbucket.org:ishikado/rust_test_bot.git


ARG LOG_LEVEL
ARG SLACK_API_TOKEN

ENV LEVEL $LOG_LEVEL
ENV TOKEN $SLACK_API_TOKEN


# fix me!!
COPY . rust_test_bot


RUN cd rust_test_bot && $HOME/.cargo/bin/cargo build
CMD ["sh", "-c", "cd rust_test_bot && $HOME/.cargo/bin/cargo run -- $TOKEN -l $LEVEL"]
