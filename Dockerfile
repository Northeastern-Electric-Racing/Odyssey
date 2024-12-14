FROM ubuntu:latest

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive TZ="America/New_York" \
    apt-get -y install tzdata


RUN apt-get update && apt-get install -y \
    make \
    binutils \
    build-essential \
    diffutils \
    gcc \
    g++ \
    bash \
    patch \
    gzip \
    bzip2 \
    perl \
    tar \
    cpio \
    unzip \
    rsync \
    file \
    bc \
    findutils \
    wget \
    python3 \
    libncurses5-dev \
    git \
    python3-matplotlib \
    graphviz \
    git-lfs \
    util-linux \ 
    wpasupplicant

# add all buildroot files there
WORKDIR /home/odysseus/build

RUN  git clone https://gitlab.com/buildroot.org/buildroot.git && cd ./buildroot && git checkout 2024.11


WORKDIR /home/odysseus/outputs/
COPY ./docker_scripts /home/odysseus/scripts
RUN echo "source /home/odysseus/scripts/setup_env.sh" >> ~/.bashrc

# install password using wildcard so failures arent deadly
COPY ./SECRETS.env.* /home/odysseus/

ENTRYPOINT "/bin/bash"
