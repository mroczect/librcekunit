BINDING_DIR := $(abspath $(CURDIR)/../..)
ROOT_DIR    := $(abspath $(BINDING_DIR)/..)
LIB_DIR     := $(ROOT_DIR)/target/release
INC_DIR     := $(BINDING_DIR)/include
HEADER      := $(INC_DIR)/librcekunit.h
LIB_FILE    := $(LIB_DIR)/liblibrcekunit.so

CC      ?= cc
CFLAGS  := -I$(INC_DIR) -Wall -Wextra -O2 -std=c11
LDFLAGS := $(LIB_FILE) -lpthread -ldl -lm
RPATH   := -Wl,-rpath,$(LIB_DIR)

main: main.c $(HEADER) $(LIB_FILE)
	$(CC) $(CFLAGS) -o $@ main.c $(LDFLAGS) $(RPATH)

$(HEADER):
	@echo "Header belum ada. Jalankan dulu dari root repo:"
	@echo "  cargo build -p librcekunit_c_binding --release"
	@exit 1

$(LIB_FILE):
	@echo "Library belum ada. Jalankan dulu dari root repo:"
	@echo "  cargo build -p librcekunit_c_binding --release"
	@exit 1

run: main
	./main

clean:
	rm -f main export.csv

.PHONY: run clean
