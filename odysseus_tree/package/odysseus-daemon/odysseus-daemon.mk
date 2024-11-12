ODYSSEUS_DAEMON_VERSION = 25d6e3c8b3f078dc17ac05b8e765e1b2accb06a0
ODYSSEUS_DAEMON_SITE_METHOD = git
ODYSSEUS_DAEMON_SITE = https://github.com/Northeastern-Electric-Racing/Odysseus-Daemon

# all dependencies and support scripts are in the TPU overlay as this package is TPU specifc

$(eval $(cargo-package))
