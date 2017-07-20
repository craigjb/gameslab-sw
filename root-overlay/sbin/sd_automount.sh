#!/bin/sh

destdir=/mnt/sdcard

sd_umount()
{
    if grep -qs "^/dev/$1 " /proc/mounts ; then
        umount "${destdir}";
    fi

    [ -d "${destdir}" ] && rmdir "${destdir}"
}

sd_mount()
{
    mkdir -p "${destdir}" || exit 1

    if ! mount -t auto -o sync "/dev/$1" "${destdir}"; then
        # failed to mount, clean up mountpoint
        rmdir "${destdir}"
        exit 1
    fi
}

case "${ACTION}" in
add|"")
    sd_umount ${MDEV}
    sd_mount ${MDEV}
;;
remove)
    sd_umount ${MDEV}
;;
esac
