use audio::ac97::AC97;
use audio::intelhda::IntelHDA;

use core::cell::UnsafeCell;

use common::debug;

use disk::ahci::Ahci;
use disk::ide::Ide;

use env::Environment;

use network::intel8254x::Intel8254x;
use network::rtl8139::Rtl8139;

use schemes::file::FileScheme;

use usb::ehci::Ehci;
use usb::ohci::Ohci;
use usb::uhci::Uhci;
use usb::xhci::Xhci;

use super::config::PciConfig;
use super::common::class::*;
use super::common::subclass::*;
use super::common::programming_interface::*;
use super::common::vendorid::*;
use super::common::deviceid::*;

/// PCI device
pub unsafe fn pci_device(env: &mut Environment,
                         mut pci: PciConfig,
                         class_id: u8,
                         subclass_id: u8,
                         interface_id: u8,
                         vendor_code: u16,
                         device_code: u16) {
    if class_id == MASS_STORAGE {
        if subclass_id == IDE {
            if let Some(module) = FileScheme::new(Ide::disks(pci)) {
                env.schemes.push(UnsafeCell::new(module));
            }
        } else if subclass_id == SATA && interface_id == AHCI {
            if let Some(module) = FileScheme::new(Ahci::disks(pci)) {
                env.schemes.push(UnsafeCell::new(module));
            }
        }
    } else if class_id == SERIAL_BUS && subclass_id == USB {
        if interface_id == XHCI {
            let base = pci.read(0x10) as usize;

            let mut module = box Xhci {
                pci: pci,
                base: base & 0xFFFFFFF0,
                memory_mapped: base & 1 == 0,
                irq: pci.read(0x3C) as u8 & 0xF,
            };
            module.init();
            env.schemes.push(UnsafeCell::new(module));
        } else if interface_id == EHCI {
            env.schemes.push(UnsafeCell::new(Ehci::new(pci)));
        } else if interface_id == OHCI {
            env.schemes.push(UnsafeCell::new(Ohci::new(pci)));
        } else if interface_id == UHCI {
            env.schemes.push(UnsafeCell::new(Uhci::new(pci)));
        } else {
            debug!("Unknown USB interface version {:X}\n", interface_id);
        }
    } else {
        match vendor_code {
            REALTEK => {
                match device_code {
                    RTL8139 => env.schemes.push(UnsafeCell::new(Rtl8139::new(pci))),
                    _ => (),
                }
            }
            INTEL => {
                match device_code {
                    GBE_82540EM => env.schemes.push(UnsafeCell::new(Intel8254x::new(pci))),
                    AC97_82801AA | AC97_ICH4 => env.schemes.push(UnsafeCell::new(AC97::new(pci))),
                    INTELHDA_ICH6 => {
                        let base = pci.read(0x10) as usize;
                        let mut module = box IntelHDA {
                            pci: pci,
                            base: base & 0xFFFFFFF0,
                            memory_mapped: base & 1 == 0,
                            irq: pci.read(0x3C) as u8 & 0xF,
                        };
                        module.init();
                        env.schemes.push(UnsafeCell::new(module));
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
}

/// Initialize PCI session
pub unsafe fn pci_init(env: &mut Environment) {
    for bus in 0..256 {
        for slot in 0..32 {
            for func in 0..8 {
                let mut pci = PciConfig::new(bus as u8, slot as u8, func as u8);
                let id = pci.read(0);

                if (id & 0xFFFF) != 0xFFFF {
                    let class_id = pci.read(8);

                    debug!(" * PCI {}, {}, {}: ID {:X} CL {:X}",
                           bus,
                           slot,
                           func,
                           id,
                           class_id);

                    for i in 0..6 {
                        let bar = pci.read(i * 4 + 0x10);
                        if bar > 0 {
                            debug!(" BAR{}: {:X}", i, bar);

                            pci.write(i * 4 + 0x10, 0xFFFFFFFF);
                            let size = (0xFFFFFFFF - (pci.read(i * 4 + 0x10) & 0xFFFFFFF0)) + 1;
                            pci.write(i * 4 + 0x10, bar);

                            if size > 0 {
                                debug!(" {}", size);
                            }
                        }
                    }

                    debug::dl();

                    pci_device(env,
                               pci,
                               ((class_id >> 24) & 0xFF) as u8,
                               ((class_id >> 16) & 0xFF) as u8,
                               ((class_id >> 8) & 0xFF) as u8,
                               (id & 0xFFFF) as u16,
                               ((id >> 16) & 0xFFFF) as u16);
                }
            }
        }
    }
}
