#![no_main]
#![no_std]

use arrayvec::ArrayString;
use core::fmt::Write;
use hd44780_driver::HD44780;
use panic_rtt_target as _;

use cortex_m_rt::entry;
use rtt_target::{rprint, rprintln};
use stm32l4xx_hal::{
    adc::{Adc, AdcCommon},
    delay::Delay,
    pac,
    prelude::*,
};

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();
    rprint!("Initializing...");

    let cp = pac::CorePeripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    let clocks = rcc.cfgr.freeze(&mut flash.acr, &mut pwr);

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);

    // LCD Buttons
    let adc_common = AdcCommon::new(dp.ADC_COMMON, &mut rcc.ahb2);
    let mut adc = Adc::adc1(dp.ADC1, adc_common, &mut rcc.ccipr, &mut delay);
    let mut a2 = gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr);

    // LCD
    let mut lcd = HD44780::new_4bit(
        gpioa
            .pa9
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper), // Register Select pin
        gpioc
            .pc7
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper), // Enable pin
        gpiob
            .pb5
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper), // d4
        gpiob
            .pb4
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper), // d5
        gpiob
            .pb10
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper), // d6
        gpioa
            .pa8
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper), // d7
        &mut delay,
    )
    .unwrap();

    lcd.reset(&mut delay).unwrap();
    lcd.clear(&mut delay).unwrap();
    lcd.set_cursor_visibility(hd44780_driver::Cursor::Invisible, &mut delay)
        .unwrap();
    lcd.set_cursor_blink(hd44780_driver::CursorBlink::Off, &mut delay)
        .unwrap();

    // Display Buffer
    let mut buf = ArrayString::<16>::new();

    rprintln!(" done.");

    loop {
        let value = adc.read(&mut a2).unwrap();
        core::write!(buf, "Reading: {}", value).unwrap();
        rprintln!("Value: {}", value);

        // Printing to LCD
        lcd.set_cursor_pos(0, &mut delay).unwrap();
        lcd.clear(&mut delay).unwrap();
        lcd.write_str(&buf, &mut delay).unwrap();

        // Clearing Buffer
        buf.clear();

        delay.delay_ms(500 as u16);
    }
}
