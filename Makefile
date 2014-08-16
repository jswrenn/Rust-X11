RUSTC = rustc
DOCGEN = rustdoc
SRC_DIR = src
SRC_FILES = X11.rs
BUILD_DIR = build
OPT_LEVEL = 3
DEBUG = true
RUST_FLAGS = --opt-level=$(OPT_LEVEL)
ifeq ($(DEBUG),false)
    RUST_FLAGS += --cfg ndebug
endif
SRC_BUILD_CMD = $(foreach file, $(SRC_FILES), $(RUSTC) $(SRC_DIR)/$(file) $(RUST_FLAGS) --out-dir=$(BUILD_DIR);)
DOC_BUILD_CMD = $(foreach file, $(SRC_FILES), $(DOCGEN) $(SRC_DIR)/$(file);)

all: options src doc

src:
	$(SRC_BUILD_CMD)

doc:
	$(DOC_BUILD_CMD)
options:
	@echo "RUSTC         = ${RUSTC}"
	@echo "DOCGEN        = ${DOCGEN}"
	@echo "SRC_DIR       = ${SRC_DIR}"
	@echo "SRC_FILES     = ${SRC_FILES}"
	@echo "BUILD_DIR     = ${BUILD_DIR}"
	@echo "DEBUG         = ${DEBUG}"
	@echo "OPT_LEVEL     = ${OPT_LEVEL}"
	@echo "RUST_FLAGS    = ${RUST_FLAGS}"
	@echo "SRC_BUILD_CMD = ${SRC_BUILD_CMD}"
	@echo "DOC_BUILD_CMD = ${DOC_BUILD_CMD}"

clean:

.PHONY: all src doc options clean
