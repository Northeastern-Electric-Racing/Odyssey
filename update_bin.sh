#!/bin/sh


# 1: The name of the package to update
# 2: The root password of the TPU
# 3: The commit sha (only if calypso)

# 1: The name of the package in buildroot
update_pkg_br() {
   docker_hash=$(docker compose run -d --rm odysseus)
   docker exec "$docker_hash" sh -c "make -C /home/odysseus/outputs/tpu $1-reconfigure"
   docker stop "$docker_hash"
}

# 1: The root password of the TPU
# 2: The name of the initscript to stop on the target
# 3: The path to the binary local to send to target
# 4: The path to put on target
refresh_local() {
   sshpass -p "$1" ssh root@192.168.100.12 -t "/etc/init.d/$2 stop"
   sshpass -p "$1" scp "$3" "root@192.168.100.12:$4"
   sshpass -p "$1" ssh root@192.168.100.12 -t "/etc/init.d/$2 start"
}


if [ "$1" == "calypso" ]; then
   echo "Updating Calypso"
   sed -i "1 s/.*/CALYPSO_VERSION = $3/" ./odysseus_tree/package/calypso/calypso.mk
   update_pkg_br "calypso"
   refresh_local "$2" "S76calypso" "./outputs/tpu/per-package/calypso/target/usr/bin/calypso" "/usr/bin/calypso"
elif [ "$1" == "nero" ]; then
   echo "Updating NERO"
   update_pkg_br "nero2"
   refresh_local "$2" "S99nero2" "./outputs/tpu/per-package/nero2/target/usr/bin/NEROApp" "/usr/bin/NEROApp"
else
   echo "Not a valid project to update"
fi

