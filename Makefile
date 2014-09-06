RUSTC = rustc
DOCGEN = rustdoc
LIB_DIR = lib
LOCAL_LIB_FILES = syntax_extensions.rs
SRC_DIR = src
SRC_FILES = X11.rs
BUILD_DIR = build
DEBUG = true
OPT_LEVEL = 3
LINK_FLAGS = -L $(BUILD_DIR)
RUST_FLAGS = --opt-level=$(OPT_LEVEL)
ifeq ($(DEBUG),false)
    RUST_FLAGS += --cfg ndebug
endif
LOCAL_LIBS_BUILD_CMD = $(foreach file, $(LOCAL_LIB_FILES), $(RUSTC) $(RUST_FLAGS) $(LIB_DIR)/$(file) --out-dir=$(BUILD_DIR);)
SRC_BUILD_CMD = $(foreach file, $(SRC_FILES), $(RUSTC) $(LINK_FLAGS) $(SRC_DIR)/$(file) $(RUST_FLAGS) --out-dir=$(BUILD_DIR);)
DOC_BUILD_CMD = $(foreach file, $(SRC_FILES), $(DOCGEN) $(LINK_FLAGS) $(SRC_DIR)/$(file);)

all: options local_libs src doc

local_libs:
	$(LOCAL_LIBS_BUILD_CMD)

src:
	$(SRC_BUILD_CMD)

doc:
	$(DOC_BUILD_CMD)
options:
	@echo "RUSTC                = ${RUSTC}"
	@echo "DOCGEN               = ${DOCGEN}"
	@echo "LIB_DIR              = ${LIB_DIR}"
	@echo "LOCAL_LIB_FILES      = ${LOCAL_LIB_FILES}"
	@echo "SRC_DIR              = ${SRC_DIR}"
	@echo "SRC_FILES            = ${SRC_FILES}"
	@echo "BUILD_DIR            = ${BUILD_DIR}"
	@echo "DEBUG                = ${DEBUG}"
	@echo "LINK_FLAGS           = ${LINK_FLAGS}"
	@echo "OPT_LEVEL            = ${OPT_LEVEL}"
	@echo "RUST_FLAGS           = ${RUST_FLAGS}"
	@echo "LOCAL_LIBS_BUILD_CMD = ${LOCAL_LIBS_BUILD_CMD}"
	@echo "SRC_BUILD_CMD        = ${SRC_BUILD_CMD}"
	@echo "DOC_BUILD_CMD        = ${DOC_BUILD_CMD}"

clean:

.PHONY: all src doc options clean
