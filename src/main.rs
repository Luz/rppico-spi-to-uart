// Hardware: Raspberry Pi Pico
#![no_std]
#![no_main]

use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;
use format_no_std::show;
use hal::fugit::RateExtU32;
use hal::uart::{DataBits, StopBits, UartConfig};
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::clocks::init_clocks_and_plls;
use rp_pico::hal::pac;
use rp_pico::hal::prelude::*;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let sio = hal::Sio::new(pac.SIO);

    // Configures the system clock to 125 MHz
    let clocks = init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Setup our pins
    let spi_mosi = pins.gpio7.into_function::<hal::gpio::FunctionSpi>();
    let spi_miso = pins.gpio4.into_function::<hal::gpio::FunctionSpi>();
    let spi_sclk = pins.gpio6.into_function::<hal::gpio::FunctionSpi>();
    let spi = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk));

    // Exchange the uninitialised SPI driver for an initialised one
    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16.MHz(),
        embedded_hal::spi::MODE_0,
    );

    let uart_pins = (
        // Uart Tx on pin 1 (GPIO0)
        pins.gpio0.into_function::<hal::gpio::FunctionUart>(),
        // Uart Rx on pin 2 (GPIO1)
        pins.gpio1.into_function::<hal::gpio::FunctionUart>(),
    );

    let uart = hal::uart::UartPeripheral::<_, _, _>::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            // hal::uart::common_configs::_115200_8_N_1,
            UartConfig::new(115200_u32.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let mut led = pins.led.into_push_pull_output();

    let mut in_buffer = [0u8; 64];
    let mut out_buffer = [0u8; 64];

    let mut format_output = 0; // decimal

    loop {
        let numbers_received = uart.read_raw(&mut in_buffer).unwrap_or(0);
        match numbers_received {
            0 => (), // Generally do nothing
            1 => {
                // If exactly one byte is received
                match in_buffer[0] {
                    b'd' => format_output = 0, // 'd' for decimal
                    b'f' => format_output = 1, // 'f' for float
                    _ => (),
                }
            }
            _ => {
                let s = show(
                    &mut out_buffer,
                    format_args!("Pico received {} bytes!", numbers_received),
                )
                .unwrap();
                uart.write_full_blocking(s.as_bytes());
                uart.write_full_blocking(b"\n");
            }
        }

        let mut spi_buffer: [u8; 4] = [1, 2, 3, 4];
        let read_success = spi.read(&mut spi_buffer);
        match read_success {
            Ok(_) => {}  // TODO Handle success
            Err(_) => {} // TODO handle errors
        };

        match format_output {
            0 => {
                uart.write_full_blocking(&spi_buffer);
            }
            _ => {
                let s = show(
                    &mut out_buffer,
                    format_args!(
                        "Bits: {:08b} {:08b} {:08b} {:08b}",
                        &spi_buffer[0], &spi_buffer[1], &spi_buffer[2], &spi_buffer[3]
                    ),
                )
                .unwrap();
                uart.write_full_blocking(s.as_bytes());
            }
        }
        uart.write_full_blocking(b"\n");

        led.set_high().unwrap();
        delay.delay_ms(20);
        led.set_low().unwrap();
        delay.delay_ms(20);
    }
}
