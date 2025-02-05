#include <stdint.h>
#include "hw/sysbus.h"
#include "hw/i2c/i2c.h"
#include "qom/object.h"
                                                                        
#define TYPE_TWI_I2C "TWI_I2C"
OBJECT_DECLARE_SIMPLE_TYPE(TWI_I2CState, TWI_I2C)

struct TWI_I2CState {
    SysBusDevice parent_obj;
    MemoryRegion iomem;
    char someprop;
    I2CBus *bus;
    qemu_irq irq;
    int8_t twcr;
    int8_t twsr;
    int8_t twdr;
    // TODO: why is the size incorrect? what am i missing here. Get cbindgen to work.
    char _[1000];
};
