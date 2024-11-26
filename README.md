# [Odysseus](https://nerdocs.atlassian.net/wiki/spaces/NER/pages/107184222/Odysseus)
Our custom buildroot-based OS.  It enables the collection, translation, and transportation of data from the car to a base station which hosts data visualization and analytics systems as part of the [Odyssey 22A project](https://nerdocs.atlassian.net/wiki/spaces/NER/pages/105283597/Wireless+22A+Software+Design).


Odysseus code holds the structure and configuration to enable the Wireless and Siren projects, which are summarized below:

## [Wireless](https://nerdocs.atlassian.net/wiki/spaces/NER/pages/71631135/Wireless+22A)
Our HaLow Wifi implementation and usage for low throughput high range TCP/IP data transmission.  HaLow (802.11ah) is a new 900 mhz unlicensed band wifi protocol which boasts an ultra-long range and low power usage.  This repository contains the buildroot enablement for the protocol with Newracom chips.

## [Siren](https://nerdocs.atlassian.net/wiki/spaces/NER/pages/107151426/Siren)
Siren is our [pub/sub](https://www.stackpath.com/edge-academy/what-is-pub-sub-messaging/) server that uses a MQTT server to send telemetry data from the car. Siren is a custom [Mosquitto](https://mosquitto.org) server.  Configuration code for mosquitto on the car lives in the rootfs overlays of buildroot, and for the base station it lies in the `extra` folder.

### About MQTT
For information about MQTT, check out [this confluence page](https://nerdocs.atlassian.net/wiki/spaces/NER/pages/173113345/Delving+into+MQTT).

### Running with Docker
Custom image coming soon. For now, you can run with the instructions in the `extra/mosquitto_base` folder, and to achieve in-car configuration use the image but substitute the configurations with those found in the buildroot rootfs overlay for the TPU.  More developer side Siren usage is found in the Argos repository.

### Local Setup
Docker is easiest, commands are in `extra/mosquitto_base`.  If you would like to install locally, visit the mosquitto website to learn more.

### Testing Siren
To test that Siren is working properly, run the `subscriber.py` and `publisher.py` scripts in the `extra/siren_example` folder in the same environment that Siren is hosted in. After a few seconds, the terminal running the `subscriber.py` script should begin receiving messages, which means that Siren is working properly.


## Building Odysseus


### Current configurations (see below for how to build)
- `raspberrypi4_64_tpu_defconfig` for TPU
    - NRC HaLow station
    - NanoMQ MQTT
    - Calypso CAN decoding
    - GPS support
    - Docker
- `raspberrypi3_64_iroh_defconfig` for off-car charging module
    - Calypso CAN decoding
    - 2.4/5ghz wireless connectivity to base station
- `raspberrypi3_64_ap_defconfig` for base station HaLow access point
    - NRC HaLow access point

All defconfigs come with (in addition to busybox and util_linux utilities):

- SSH server/client (and scp client)
- SFTP server (scp server support)
- htop
- bmon
- fsck
- python3
- GPIO read/write utilities
- dtoverlay support
- iperf3, iw, iputils, and other network configuration utilities

## Quick start
Download and install to PATH git and docker.  Docker desktop is required for macOS and Windows. Windows users must use WSL, and have WSL integration enabled in Docker Desktop --> Settings --> Resources --> WSL Integration --> Ubuntu (or other).  
Then follow these steps in the terminal (WSL users must not be in the mnt/c directory!).  
All platforms:
```
git clone https://github.com/Northeastern-Electric-Racing/Odysseus.git # or use SSH if configured
cd ./Odysseus
git submodule update --init --recursive
```

Now launch the docker command line for Odysseus:

For Linux and WSL:
```
docker compose run --rm odysseus
```

For mac: (experimental, limited):
```
docker compose -f "compose-compat.yml" run  --rm --build odysseus # Future launches can omit `--build` for time savings and space savings, but it should be used if the Dockerfile or docker_out_of_tree.sh files change.  
```


Now you are in the docker container.  To build cd into the defconfig directory (ap, tpu, etc), then run the make command alias:
```
cd ./<defconfig>
make-current
```

**Note: If failure occurs very early in the process, run `make nanomq-source` and try again before reporting the error.**
You can view the `output.log` for more info.

### More on docker configuration
The container has a directory structure as so:
(everything is in `/home/odysseus)
- `./build`
    - `./buildroot`: The buildroot tree 
    - `./odysseus_tree`: The odyssues external tree, bound to the same directory in the git repository on your local machine!
- `./shared_data`: The download and ccache cache for buildroot, should be persisted as long as space is available, there is usually no reason to enter this. A persistent docker volume with the name   `odysseus_shared_data`.
- `./outputs/*`:
    - **The output folders for odysseus.  `cd` into the one named for what defconfig you would like to build, and run the `make` configuration and build commands as described below.  It is recommended to save space to run `make clean` in defconfig directories rather than removing this volume all together. For Linux hosts, this is bound to the `./odysseus/outputs` directory in the repository. For Windows users, this is a docker volume.  *Remember to use `make savedefconfig` when you are done as changes are overriden when you re-open the docker image!*

### Extra docker tips
All paths relative to Siren root.

#### Note for macOS users
The outputs are stored in a docker volume on these platforms to ensure rsync compatability.  Therefore fetching the files requires first running the image, then use `docker cp` to get them to the userspace.  Alternatively, docker desktop has a file explorer for docker volumes that may come in handy.

#### Writing the sd card
The image is present in `./outputs/<defconfig name>/images/sdcard.img`.  One can flash this with tools like the Raspberry Pi OS Imager.

#### Pulling source files for scp, etc.
The target binaries are located in `./outputs/<defconfig name>/target`.

#### Cleaning system
`docker image prune --all` (this will not touch volumes)

**IMPORTANT**: this WILL wipe all shared cache (don't do this unless you need the space):  
`docker volume rm odysseus_shared_data`

This will wipe all outputs, not the cache so just requires a full rebuild to recover from.  On compose-compat images only:
`docker volume rm odysseus_outputs`


#### Open another tty
`docker ps`
Find the container ID of odysseus-odysseus then run:  
`docker exec -it <container_id> bash` 

#### Run in background
One can still build, but in the background.  This can be done by using docker-compose with `-d` and running docker exec like so:
```
docker exec <container_id> -d -w /home/odysseus/outputs/<defconfig> make-current`
```
Be careful with this.

### Passwords
Root passwords are stored via Github secrets and an encrypted file within a ghcr docker image. To load them, run `load-secrets` in the docker compose image (pre-built) and enter the master password.  Consult Odysseus lead if you need this info.  Now your passwords are loaded (can be viewed with `env | grep ODY`), and will be set when you make the sdcard.img.  Note this step must be repeated on each `docker compose run --rm odysseus`, and if the passwords change on Github steps 2 and 3 must be rerun as well.



See below to learn more about developing, and check confluence for most info.  Once in the docker image, all the normal make commands (in an out-of-tree context only) apply.

## Configuring the Project
1. Run ```make menuconfig``` after initializing
2. Make any customizations you want in the menu
3. Save changes after you've made them by running ```make savedefconfig```.  Ensure you are saving changes to the intended defconfig, it is saved to whatever directory you `cd`ed into!

#### Adding defconfigs

Checklist when adding a defconfig:  
[ ] Add a secret for SSH password in settings, and load it into the workflow env in `.github/workflows/build_image.yml`  
[ ] Add the defconfig itself, changing path names, etc.  Board folders and overlays can still be shared as needed.  
[ ] Add a pretty name to `post-build-os-release.sh`  
[ ] Add a load command to `setup_env.sh`
