TARGET_DIR ?= target
RELEASE_DIR ?= $(TARGET_DIR)/thumbv6m-none-eabi/release
FQBN ?= arduino:samd:nano_33_iot
SERIAL ?= /dev/ttyACM0
OUT ?= plant

$(RELEASE_DIR)/$(OUT): $(wildcard src/**/*)
	cargo build --release

$(TARGET_DIR)/$(OUT).bin: $(RELEASE_DIR)/$(OUT)
	arm-none-eabi-objcopy -O binary $(RELEASE_DIR)/$(OUT) $(TARGET_DIR)/$(OUT).bin

.PHONY: deploy
deploy: $(TARGET_DIR)/$(OUT).bin
	arduino-cli upload -i $(TARGET_DIR)/$(OUT).bin -b $(FQBN) -p $(SERIAL)

.PHONY: clean
clean:
	rm $(TARGET_DIR)/$(OUT).bin
	cargo clean
