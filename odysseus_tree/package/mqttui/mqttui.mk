MQTTUI_VERSION = 0.21.1
MQTTUI_SITE = $(call github,Northeastern-Electric-Racing,mqttui,v$(MQTTUI_VERSION)-ner)
MQTTUI_LICENSE = GPL-3.0+

$(eval $(cargo-package))
