#!/bin/bash
                                                                                                                                      
alias make-current="/home/odysseus/scripts/make-current.sh"
alias load-secrets="source /home/odysseus/scripts/load-secrets.sh"

# for each defconfig make output subdirectory 
make -C /home/odysseus/build/buildroot O=/home/odysseus/outputs/tpu BR2_EXTERNAL=/home/odysseus/build/odysseus_tree raspberrypi4_64_tpu_defconfig
make -C /home/odysseus/build/buildroot O=/home/odysseus/outputs/ap-pi3 BR2_EXTERNAL=/home/odysseus/build/odysseus_tree raspberrypi3_64_ap_defconfig
make -C /home/odysseus/build/buildroot O=/home/odysseus/outputs/ap-pi4 BR2_EXTERNAL=/home/odysseus/build/odysseus_tree raspberrypi4_64_ap_defconfig
make -C /home/odysseus/build/buildroot O=/home/odysseus/outputs/iroh BR2_EXTERNAL=/home/odysseus/build/odysseus_tree raspberrypi3_64_iroh_defconfig


cat << "EOF"
 .d88b.  d8888b. db    db .d8888. .d8888. d88888b db    db .d8888. 
.8P  Y8. 88  `8D `8b  d8' 88'  YP 88'  YP 88'     88    88 88'  YP 
88    88 88   88  `8bd8'  `8bo.   `8bo.   88ooooo 88    88 `8bo.   
88    88 88   88    88      `Y8b.   `Y8b. 88      88    88   `Y8b. 
`8b  d8' 88  .8D    88    db   8D db   8D 88.     88b  d88 db   8D 
 `Y88P'  Y8888D'    YP    `8888Y' `8888Y' Y88888P ~Y8888P' `8888Y'

EOF
