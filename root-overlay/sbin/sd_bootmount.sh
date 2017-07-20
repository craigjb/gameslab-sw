#!/bin/sh

destdir=/mnt/sdcard
device=/dev/mmcblk0p1

mkdir -p "${destdir}" || exit 1

if ! mount -t auto -o sync "${device}" "${destdir}"; then
    # failed to mount, clean up mountpoint
    rmdir "${destdir}"
    exit 1
fi
