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

# fix me!!
COPY . rust_test_bot

ARG SLACK_API_TOKEN
RUN cd rust_test_bot && $HOME/.cargo/bin/cargo build

#CMD cd rust_test_bot && $HOME/.cargo/bin/cargo run -- $SLACK_API_TOKEN -l debug
