// ymf271_ffi.c — FFI shim exposing the libvgm YMF271 static functions to Rust.
//
// All callable functions in ymf271.c are declared static, making them
// inaccessible from other translation units. This file gains access by
// #including ymf271.c directly, then re-exports the required subset under
// stable extern-C names. Only ymf271_ffi.c (not ymf271.c) is compiled.
//
// License: BSD-3-Clause (follows ymf271.c)

#include <string.h>  // memset
#include "ymf271.c"

void* ymf271_ffi_create(unsigned int clock)
{
    DEV_GEN_CFG cfg;
    memset(&cfg, 0, sizeof(cfg));
    cfg.clock = clock;

    DEV_INFO devInfo;
    memset(&devInfo, 0, sizeof(devInfo));

    if (device_start_ymf271(&cfg, &devInfo) != 0x00)
        return (void*)0;

    // ymf271_update skips every group when mem_base == NULL, including FM groups.
    // Allocate a minimal 1-byte ROM so FM output is not gated by the PCM ROM check.
    void* chip_ptr = devInfo.dataPtr->chipInf;
    ymf271_alloc_rom(chip_ptr, 1);
    return chip_ptr;
}

void ymf271_ffi_destroy(void* chip)
{
    device_stop_ymf271(chip);
}

void ymf271_ffi_reset(void* chip)
{
    device_reset_ymf271(chip);
}

void ymf271_ffi_write(void* chip, unsigned char offset, unsigned char data)
{
    ymf271_w(chip, offset, data);
}

void ymf271_ffi_update(void* chip, unsigned int samples,
                       int* left, int* right)
{
    DEV_SMPL* outputs[2];
    outputs[0] = (DEV_SMPL*)left;
    outputs[1] = (DEV_SMPL*)right;
    ymf271_update(chip, samples, outputs);
}

void ymf271_ffi_alloc_rom(void* chip, unsigned int rom_size)
{
    ymf271_alloc_rom(chip, rom_size);
}

void ymf271_ffi_write_rom(void* chip, unsigned int offset,
                          unsigned int length, const unsigned char* data)
{
    ymf271_write_rom(chip, offset, length, data);
}
