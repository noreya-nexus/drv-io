#!/bin/bash
set -e

APP_VERSION="$(/usr/bin/python3 getversion.py)"

VERSION="$APP_VERSION"
DEB_VERSION="1"
PACKAGE="nexus-drv-io"
PACKAGE_NAME="$PACKAGE"_"$VERSION"
YEAR=$(date +"%Y")
export DEBFULLNAME="Unofficial"
export DEBEMAIL="nobody@example.com"

# Clean and create build dir
rm -Rf builddeb/ || true
mkdir builddeb/
cd builddeb/


mkdir -p "$PACKAGE_NAME/lib/systemd/system/"
mkdir -p "$PACKAGE_NAME/usr/bin/"

cp ../"$PACKAGE".service "$PACKAGE_NAME/lib/systemd/system/"
if [ -f "../target/release/$PACKAGE" ]; then
  cp ../target/release/$PACKAGE "$PACKAGE_NAME/usr/bin/"
else
  cp ../target/"$(arch)"-unknown-linux-gnu/release/$PACKAGE "$PACKAGE_NAME/usr/bin/"
fi

cd "$PACKAGE_NAME"

# Create manifest
mkdir debian
dch --create -v "$VERSION"-"$DEB_VERSION" --package "$PACKAGE" --distribution stable "Please visit our website for details!"
echo "10" > ./debian/compat

echo "Source: $PACKAGE" > ./debian/control
echo "Maintainer: $DEBFULLNAME <$DEBEMAIL>" >> ./debian/control
echo "Section: misc" >> ./debian/control
echo "Priority: optional" >> ./debian/control
echo "Standards-Version: 3.9.2" >> ./debian/control
echo "Build-Depends: debhelper (>= 10)" >> ./debian/control
echo "" >> ./debian/control 
echo "Package: $PACKAGE" >> ./debian/control 
echo "Architecture: any" >> ./debian/control
echo "Depends: \${misc:Depends}, \${shlibs:Depends}, bash (>= 2.1.0), systemd, nexus-power-mgmt, nexus-udev-sdbpk, libudev1" >> ./debian/control
echo "Description: IO Module driver" >> ./debian/control
echo "	Provides an UDS socket API." >> ./debian/control

# Copyright
echo "Copyright $YEAR $DEBFULLNAME $DEBEMAIL" > ./debian/copyright
echo "Licenced under /usr/share/common-licenses/GPL-3" >> ./debian/copyright

# Rules
echo "#!/usr/bin/make -f" >>  ./debian/rules
echo "" >> ./debian/rules
echo "export DH_VERBOSE=1" >> ./debian/rules
echo "" >> ./debian/rules
echo "%:" >>  ./debian/rules
echo -e "\tdh \$@ --with-systemd --no-automatic-dbgsym" >>  ./debian/rules

# Deb format
mkdir ./debian/source
echo "3.0 (quilt)" > ./debian/source/format

# Pre install script
echo "#!/bin/bash -e" >> ./debian/preinst
echo "#DEBHELPER#" >> ./debian/preinst

# Install script
echo "lib/* lib/" >> ./debian/install
echo "usr/* usr/" >> ./debian/install

# Post install script
echo "#!/bin/bash -e" >> ./debian/postinst
echo "#DEBHELPER#" >> ./debian/postinst

# Pre removal script
echo "#!/bin/bash -e" >> ./debian/prerm
echo "#DEBHELPER#" >> ./debian/prerm

# Post removal script
echo "#!/bin/bash -e" >> ./debian/postrm
echo "#DEBHELPER#" >> ./debian/postrm

#cd "$PACKAGE_NAME"
debuild -b -uc -us --no-sign
