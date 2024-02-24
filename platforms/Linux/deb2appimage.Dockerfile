FROM ubuntu

RUN apt update
RUN apt-get install -y wget file gpg appstream
RUN wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
RUN chmod +x appimagetool-x86_64.AppImage
RUN ./appimagetool-x86_64.AppImage --appimage-extract
COPY webrogue.deb webrogue.deb
RUN sh squashfs-root/AppRun --help
COPY TemplateAppDir AppDir
RUN chmod +x AppDir/AppRun
RUN dpkg-deb -R webrogue.deb webrogue_deb
RUN cp webrogue_deb/usr/bin/webrogue AppDir/webrogue
RUN cp -r webrogue_deb/usr/share/webrogue/mods AppDir/mods
RUN ls -R webrogue_deb
RUN ARCH=x86_64 sh squashfs-root/AppRun AppDir
