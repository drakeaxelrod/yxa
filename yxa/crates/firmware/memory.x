/* STM32F401CCU6 Memory Layout
 *
 * Flash: 256KB (0x40000)
 * RAM: 64KB (0x10000)
 *
 * Using native DFU bootloader (no PlumBL), so we start at 0x08000000
 */

MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K
  RAM   : ORIGIN = 0x20000000, LENGTH = 64K
}
