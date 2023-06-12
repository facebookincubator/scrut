UNAME := $(shell uname)
BUILD_TARGET := debug
BUILD_OUT := out

ifeq ($(BUILD_TARGET), release)
BUILD_FLAG = --release
else
BUILD_FLAG =
endif

ifeq ($(UNAME), Darwin)
TARGET_FROM = target/$(BUILD_TARGET)/libpyscrut.dylib
TARGET_TO = $(BUILD_OUT)/pyscrut.so
else ifeq ($(UNAME), Linux)
TARGET_FROM = target/$(BUILD_TARGET)/libpyscrut.dylib
TARGET_TO = $(BUILD_OUT)/pyscrut.dylib
else
TARGET_FROM = target/$(BUILD_TARGET)/libpyscrut.dll
TARGET_TO = $(BUILD_OUT)/pyscrut.pyd
endif

clean:
	[ -f $(TARGET_TO) ] && rm -f "$(TARGET_TO)"

build: $(TARGET_TO)
	@echo Done

test: $(TARGET_TO)
	@PYTHONPATH=$(shell dirname "$(TARGET_TO)") \
		python3 -m unittest discover -s pytest -p "test_*.py"

$(TARGET_TO): $(TARGET_FROM) $(BUILD_OUT)
	@cp -av "$(TARGET_FROM)" "$(TARGET_TO)"

$(TARGET_FROM): $(shell find ./ -type f -name "*.rs") $(shell find ../src/ -type f -name "*.rs")
	maturin build $(BUILD_FLAG)

$(BUILD_OUT):
	@mkdir -p "$@"


.PHONY: build test
