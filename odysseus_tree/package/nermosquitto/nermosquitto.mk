################################################################################
#
# nermosquitto
# NER changes: refactor MOSQUITTO --> NERMOSQUITTO
# all olther changes suffixed with # ***
################################################################################

NERMOSQUITTO_VERSION = 2.0.20
NERMOSQUITTO_SITE = https://mosquitto.org/files/source
NERMOSQUITTO_SOURCE = mosquitto-$(NERMOSQUITTO_VERSION).tar.gz # ***
NERMOSQUITTO_LICENSE = EPL-2.0 or EDLv1.0
NERMOSQUITTO_LICENSE_FILES = LICENSE.txt epl-v20 edl-v10
NERMOSQUITTO_CPE_ID_VENDOR = eclipse
NERMOSQUITTO_INSTALL_STAGING = YES

NERMOSQUITTO_MAKE_OPTS = \
	CLIENT_STATIC_LDADD="$(NERMOSQUITTO_STATIC_LIBS)" \
	UNAME=Linux \
	STRIP=true \
	prefix=/usr \
	WITH_WRAP=no \
	WITH_DOCS=no

ifeq ($(BR2_SHARED_LIBS),y)
NERMOSQUITTO_MAKE_OPTS += WITH_STATIC_LIBRARIES=no
else
NERMOSQUITTO_MAKE_OPTS += WITH_STATIC_LIBRARIES=yes
endif

ifeq ($(BR2_STATIC_LIBS),y)
NERMOSQUITTO_MAKE_OPTS += WITH_SHARED_LIBRARIES=no
else
NERMOSQUITTO_MAKE_OPTS += WITH_SHARED_LIBRARIES=yes
endif

ifeq ($(BR2_PACKAGE_SYSTEMD),y)
NERMOSQUITTO_MAKE_OPTS += WITH_SYSTEMD=yes
NERMOSQUITTO_DEPENDENCIES += systemd
endif

# adns uses getaddrinfo_a
ifeq ($(BR2_TOOLCHAIN_USES_GLIBC),y)
NERMOSQUITTO_MAKE_OPTS += WITH_ADNS=yes
else
NERMOSQUITTO_MAKE_OPTS += WITH_ADNS=no
endif

# threaded API uses pthread_setname_np
ifeq ($(BR2_TOOLCHAIN_HAS_THREADS_NPTL),y)
NERMOSQUITTO_MAKE_OPTS += WITH_THREADING=yes
else
NERMOSQUITTO_MAKE_OPTS += WITH_THREADING=no
endif

ifeq ($(BR2_PACKAGE_OPENSSL),y)
NERMOSQUITTO_DEPENDENCIES += host-pkgconf openssl
NERMOSQUITTO_MAKE_OPTS += WITH_TLS=yes
NERMOSQUITTO_STATIC_LIBS += `$(PKG_CONFIG_HOST_BINARY) --libs openssl`
else
NERMOSQUITTO_MAKE_OPTS += WITH_TLS=no
endif

ifeq ($(BR2_PACKAGE_CJSON),y)
NERMOSQUITTO_DEPENDENCIES += cjson
NERMOSQUITTO_MAKE_OPTS += WITH_CJSON=yes
NERMOSQUITTO_STATIC_LIBS += -lcjson
else
NERMOSQUITTO_MAKE_OPTS += WITH_CJSON=no
endif

ifeq ($(BR2_PACKAGE_C_ARES),y)
NERMOSQUITTO_DEPENDENCIES += c-ares
NERMOSQUITTO_MAKE_OPTS += WITH_SRV=yes
else
NERMOSQUITTO_MAKE_OPTS += WITH_SRV=no
endif

ifeq ($(BR2_PACKAGE_LIBWEBSOCKETS),y)
NERMOSQUITTO_DEPENDENCIES += libwebsockets
NERMOSQUITTO_MAKE_OPTS += WITH_WEBSOCKETS=yes
else
NERMOSQUITTO_MAKE_OPTS += WITH_WEBSOCKETS=no
endif

# C++ support is only used to create a wrapper library
ifneq ($(BR2_INSTALL_LIBSTDCPP),y)
define NERMOSQUITTO_DISABLE_CPP
	$(SED) '/-C cpp/d' $(@D)/lib/Makefile
endef

NERMOSQUITTO_POST_PATCH_HOOKS += NERMOSQUITTO_DISABLE_CPP
endif

NERMOSQUITTO_MAKE_DIRS = lib client
ifeq ($(BR2_PACKAGE_NERMOSQUITTO_BROKER),y)
NERMOSQUITTO_MAKE_DIRS += src
endif

ifeq ($(BR2_PACKAGE_MOSQUITTO_BROKER_DYNAMIC_SECURITY_PLUGIN),y)
MOSQUITTO_MAKE_DIRS += plugins/dynamic-security
endif

define NERMOSQUITTO_BUILD_CMDS
	$(MAKE) -C $(@D) $(TARGET_CONFIGURE_OPTS) DIRS="$(NERMOSQUITTO_MAKE_DIRS)" \
		$(NERMOSQUITTO_MAKE_OPTS)
endef

define NERMOSQUITTO_INSTALL_STAGING_CMDS
	$(MAKE) -C $(@D) $(TARGET_CONFIGURE_OPTS) DIRS="$(NERMOSQUITTO_MAKE_DIRS)" \
		$(NERMOSQUITTO_MAKE_OPTS) DESTDIR=$(STAGING_DIR) install
endef

define NERMOSQUITTO_INSTALL_TARGET_CMDS
	$(MAKE) -C $(@D) $(TARGET_CONFIGURE_OPTS) DIRS="$(NERMOSQUITTO_MAKE_DIRS)" \
		$(NERMOSQUITTO_MAKE_OPTS) DESTDIR=$(TARGET_DIR) install
	rm -f $(TARGET_DIR)/etc/mosquitto/*.example
	$(INSTALL) -D -m 0644 $(@D)/mosquitto.conf \
		$(TARGET_DIR)/etc/mosquitto/mosquitto.conf
endef

ifeq ($(BR2_PACKAGE_NERMOSQUITTO_BROKER),y)
define NERMOSQUITTO_INSTALL_INIT_SYSV
	$(INSTALL) -D -m 0755 package/mosquitto/S50mosquitto \
		$(TARGET_DIR)/etc/init.d/S50mosquitto
endef

define NERMOSQUITTO_INSTALL_INIT_SYSTEMD
	$(INSTALL) -D -m 644 $(@D)/service/systemd/mosquitto.service.notify \
		$(TARGET_DIR)/usr/lib/systemd/system/mosquitto.service
endef

define NERMOSQUITTO_USERS
	mosquitto -1 mosquitto -1 * - - - Mosquitto user
endef
endif

# *** below is new function for plugin addition
#NERMOSQUITTO_CONF_OPTS += -DENABLE_PLUGIN=ON
#NERMOSQUITTO_CONF_OPTS += -DCMAKE_EXE_LINKER_FLAGS="-export-dynamic"
define NERMOSQUITTO_PLUGIN_INSTALLATION
	$(foreach plugin,$(call qstrip,$(BR2_PACKAGE_NERMOSQUITTO_PLUGIN_LIST)), \
                $(TARGET_CC) $(NERMOSQUITTO_CPPFLAGS) -I$(STAGING_DIR)/usr/include/ -I$(@D) $(plugin) -fPIC -shared -o $(@D)/$(notdir $(basename $(plugin))).so
	)
        
	$(INSTALL) -d $(TARGET_DIR)/usr/lib/mosquitto/
	$(foreach plugin,$(call qstrip,$(BR2_PACKAGE_NERMOSQUITTO_PLUGIN_LIST)), \
		$(info, "Installing $(plugin)!") \
                $(INSTALL) -D -m 0755 $(@D)/$(notdir $(basename $(plugin))).so $(TARGET_DIR)/usr/lib/mosquitto/
                
        )
endef

NERMOSQUITTO_POST_INSTALL_TARGET_HOOKS += NERMOSQUITTO_PLUGIN_INSTALLATION
# *** ^^^^


HOST_NERMOSQUITTO_DEPENDENCIES = host-pkgconf host-openssl

HOST_NERMOSQUITTO_MAKE_OPTS = \
	$(HOST_CONFIGURE_OPTS) \
	UNAME=Linux \
	STRIP=true \
	prefix=$(HOST_DIR) \
	WITH_WRAP=no \
	WITH_DOCS=no \
	WITH_TLS=yes

define HOST_NERMOSQUITTO_BUILD_CMDS
	$(MAKE) -C $(@D)/apps/mosquitto_passwd $(HOST_NERMOSQUITTO_MAKE_OPTS)
endef

define HOST_NERMOSQUITTO_INSTALL_CMDS
	$(MAKE) -C $(@D)/apps/mosquitto_passwd $(HOST_NERMOSQUITTO_MAKE_OPTS) install
endef

$(eval $(generic-package))
$(eval $(host-generic-package))
