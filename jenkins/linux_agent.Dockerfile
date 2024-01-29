FROM jenkins/ssh-agent:latest-debian-jdk17

RUN apt-get update
RUN apt-get install -y cmake zip python3-pyparsing

RUN apt-get install -y build-essential libsdl2-dev libsdl2-ttf-dev libncurses-dev

RUN cd /home/jenkins && git clone https://github.com/emscripten-core/emsdk.git
RUN cd /home/jenkins/emsdk && ./emsdk install latest && ./emsdk activate latest
RUN bash -c "source /home/jenkins/emsdk/emsdk_env.sh && embuilder.py build freetype sdl2 harfbuzz sdl2_ttf"

RUN apt-get install -y libfl2 wget
RUN wget https://github.com/andrewwutw/build-djgpp/releases/download/v3.4/djgpp-linux64-gcc1220.tar.bz2 -O - -q | tar -xjvf - -C /usr/local/

RUN apt-get install -y sdkmanager
RUN yes | sdkmanager --licenses
RUN sdkmanager "platform-tools" "ndk;21.4.7075529" "cmake;3.22.1" "emulator;32.1.15" "tools;26.1.1" "build-tools;30.0.2" "platforms;android-31"

ENV EMSDK=/home/jenkins/emsdk

RUN apt-get install -y wine64-tools python3 msitools python3-simplejson python3-six ca-certificates
RUN ln -s /usr/bin/wine /usr/bin/wine64
ENV WINEPREFIX=/home/jenkins/.wine
RUN mkdir /home/jenkins/.wine
RUN wine64 wineboot --init && while pgrep wineserver > /dev/null; do sleep 1; done
WORKDIR /opt/clones
RUN git clone https://github.com/mstorsjo/msvc-wine.git
WORKDIR /opt/clones/msvc-wine
RUN mkdir /opt/msvc
RUN cp lowercase fixinclude install.sh vsdownload.py msvctricks.cpp /opt/msvc/
RUN cp -r wrappers /opt/msvc
WORKDIR /opt/msvc
RUN PYTHONUNBUFFERED=1 ./vsdownload.py --accept-license --dest /opt/msvc
RUN PYTHONUNBUFFERED=1 ./install.sh /opt/msvc && \
    rm lowercase fixinclude install.sh vsdownload.py && \
    rm -rf wrappers
RUN cp /opt/clones/msvc-wine/msvcenv-native.sh /opt/msvc
RUN wineserver -k; wineserver -p; wine64 wineboot
RUN apt-get -y install winbind
# RUN echo "export PATH=/opt/msvc/bin/x64:\$$PATH; wineserver -p;" >> /home/jenkins/.bashrc
# chmod a+rx /root/.wine/ --recursive
RUN chown jenkins --recursive /home/jenkins/.wine && \
    chown jenkins /home/jenkins/.wine && \
    chmod g+rx /home/jenkins/.wine --recursive

COPY keys/jenkins_rsa.pub /home/jenkins/.ssh/authorized_keys
WORKDIR /home/jenkins
