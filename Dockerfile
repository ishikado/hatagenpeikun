# syntax = docker/dockerfile:experimental

FROM ubuntu:18.04

RUN apt-get update
RUN apt-get -y install curl git 
RUN apt-get install libssl-dev
RUN curl https://sh.rustup.rs > setup.sh
RUN sh setup.sh -y

RUN . $HOME/.cargo/bin

#RUN --mount=type=secret,id=ssh,target=/root/.ssh/id_rsa git clone git@bitbucket.org:ishikado/rust_test_bot.git

ADD .ssh /root/.ssh
RUN git clone git@bitbucket.org:ishikado/rust_test_bot.git

RUN cd rust_test_bot
RUN cargo build

#CMD echo "setup finished!"
