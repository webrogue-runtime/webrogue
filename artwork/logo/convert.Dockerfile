FROM ubuntu:26.04

RUN apt update
RUN apt install -y imagemagick librsvg2-bin
COPY convert.sh .
COPY logo.svg .
WORKDIR /host_dir/artwork/logo
CMD sh convert.sh
