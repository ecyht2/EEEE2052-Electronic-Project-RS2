#![no_main]
#![no_std]

use core::{cell::RefCell, fmt::Write};
use hd44780_driver::HD44780;

use cortex_m::{interrupt::Mutex, peripheral::NVIC};
use cortex_m_rt::entry;

use panic_rtt_target as _;
use rtt_target::{rprint, rprintln};

use arrayvec::ArrayString;

use doppler_radar::{comparator::Comparator, utilities, LCDButtons, ADC};

use stm32l4xx_hal::{
    adc::{Adc, AdcCommon},
    comp::{self, Comp, CompConfig, CompDevice},
    delay::Delay,
    pac::{self, interrupt},
    prelude::*,
    timer::Timer,
};

// Global Variables
static G_COMP: Mutex<RefCell<Option<Comparator>>> = Mutex::new(RefCell::new(None));
static G_ADC: Mutex<RefCell<Option<ADC>>> = Mutex::new(RefCell::new(None));

// Constants
const _ADC_BUF_LEN: u16 = 4096;
const CLOCK_FREQUENCY: u32 = 16000;
const TRANSMITTED_FREQUENCY: f32 = 10.525e9;

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();
    rprint!("Initializing...");

    // Setting Up Peripherals
    let cp = pac::CorePeripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Setting Up Clock
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    let clocks = rcc.cfgr.freeze(&mut flash.acr, &mut pwr);

    let mut delay = Delay::new(cp.SYST, clocks);

    // Setting Up GPIO
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);

    // LCD Buttons
    let adc_common = AdcCommon::new(dp.ADC_COMMON, &mut rcc.ahb2);
    let mut button_adc = Adc::adc1(dp.ADC1, adc_common, &mut rcc.ccipr, &mut delay);
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

    // Setting Up LCD
    lcd.reset(&mut delay).unwrap();
    lcd.clear(&mut delay).unwrap();
    lcd.set_cursor_visibility(hd44780_driver::Cursor::Invisible, &mut delay)
        .unwrap();
    lcd.set_cursor_blink(hd44780_driver::CursorBlink::Off, &mut delay)
        .unwrap();

    // Comparator
    // Comparator
    let cfg = CompConfig {
        blanking: comp::BlankingSource::None,
        hyst: comp::Hysterisis::NoHysterisis,
        inmsel: comp::InvertingInput::Vref,
        inpsel: comp::NonInvertingInput::Io2,
        polarity: comp::OutputPolarity::NotInverted,
        pwrmode: comp::PowerMode::HighSpeed,
    };
    let comparator = Comp::new(CompDevice::One, cfg, &mut rcc.apb2);

    // Timer
    unsafe { NVIC::unmask(stm32l4xx_hal::stm32::Interrupt::TIM1_UP_TIM16) };
    let timer = Timer::tim16(dp.TIM16, CLOCK_FREQUENCY.Hz(), clocks, &mut rcc.apb2);

    // Comparator Struct
    let mut comp = Comparator::new(comparator, timer, CLOCK_FREQUENCY as f32);

    // Intitializing
    comp.start();
    gpiob.pb2.into_analog(&mut gpiob.moder, &mut gpiob.pupdr);

    // Moving struct to global
    cortex_m::interrupt::free(|cs| *G_COMP.borrow(cs).borrow_mut() = Some(comp));

    // ADC
    let adc = ADC::new();

    // Moving struct to global
    cortex_m::interrupt::free(|cs| *G_ADC.borrow(cs).borrow_mut() = Some(adc));

    // Display Buffer
    let mut row1 = ArrayString::<16>::new();
    let mut row2 = ArrayString::<16>::new();
    // LCD Variables
    let mut sampling_mode = LCDButtons::UP;
    let mut units_mode = LCDButtons::RIGHT;
    let mut current_frequency = 0.0;
    let mut current_speed;

    rprintln!(" done.");

    loop {
        let value = button_adc.read(&mut a2).unwrap();
        let current_button = LCDButtons::new(value).unwrap();

        // Setting Mode
        match current_button {
            LCDButtons::DOWN if sampling_mode != LCDButtons::DOWN => {
                cortex_m::interrupt::free(|cs| {
                    // Moving out comp
                    let mut comp = G_COMP.borrow(cs).replace(None).unwrap();

                    comp.start();

                    // Moving comp back
                    *G_COMP.borrow(cs).borrow_mut() = Some(comp);
                });
                cortex_m::interrupt::free(|cs| {
                    // Moving out adc
                    let mut adc = G_ADC.borrow(cs).replace(None).unwrap();

                    adc.stop();

                    // Moving adc back
                    *G_ADC.borrow(cs).borrow_mut() = Some(adc);
                });
                sampling_mode = LCDButtons::DOWN;
            }
            LCDButtons::UP if sampling_mode != LCDButtons::UP => {
                cortex_m::interrupt::free(|cs| {
                    // Moving out comp
                    let mut comp = G_COMP.borrow(cs).replace(None).unwrap();

                    comp.stop();

                    // Moving comp back
                    *G_COMP.borrow(cs).borrow_mut() = Some(comp);
                });
                cortex_m::interrupt::free(|cs| {
                    // Moving out adc
                    let mut adc = G_ADC.borrow(cs).replace(None).unwrap();

                    adc.start();

                    // Moving adc back
                    *G_ADC.borrow(cs).borrow_mut() = Some(adc);
                });
                sampling_mode = LCDButtons::UP;
            }
            LCDButtons::RIGHT if units_mode != LCDButtons::RIGHT => {
                units_mode = LCDButtons::RIGHT;
            }
            LCDButtons::LEFT if units_mode != LCDButtons::LEFT => {
                units_mode = LCDButtons::LEFT;
            }
            _ => (),
        };

        // Getting Frequency
        if sampling_mode == LCDButtons::DOWN {
            cortex_m::interrupt::free(|cs| {
                // Moving out comp
                let comp = G_COMP.borrow(cs).replace(None).unwrap();

                current_frequency = comp.calculate_frequency();

                // Moving comp back
                *G_COMP.borrow(cs).borrow_mut() = Some(comp);
            });
            // Does not work after 1000 (maybe fix?)
            core::write!(row1, "COMP f: {:<8.4}", current_frequency).unwrap_or_default();
        } else if sampling_mode == LCDButtons::UP {
            cortex_m::interrupt::free(|cs| {
                // Moving out adc
                let adc = G_ADC.borrow(cs).replace(None).unwrap();

                current_frequency = adc.calculate_frequency();

                // Moving adc back
                *G_ADC.borrow(cs).borrow_mut() = Some(adc);
            });
            // Does not work after 1000 (maybe fix?)
            core::write!(row1, "ADC f: {:<9.5}", current_frequency).unwrap_or_default();
        }

        // Calculating Speeds
        if units_mode == LCDButtons::RIGHT {
            current_speed = utilities::calculate_speed(current_frequency, TRANSMITTED_FREQUENCY);
            // Does not work after 1000 (maybe fix?)
            core::write!(row2, "kmph: {:<10.6}", current_speed).unwrap_or_default();
        } else if units_mode == LCDButtons::LEFT {
            current_speed =
                utilities::calculate_speed_mph(current_frequency, TRANSMITTED_FREQUENCY);
            // Does not work after 1000 (maybe fix?)
            core::write!(row2, "mph: {:<11.7}", current_speed).unwrap_or_default();
        }

        // Printing to LCD
        // Row 1
        lcd.set_cursor_pos(0, &mut delay).unwrap();
        lcd.write_str(&row1, &mut delay).unwrap();
        // Row 2
        lcd.set_cursor_pos(40, &mut delay).unwrap();
        lcd.write_str(&row2, &mut delay).unwrap();

        // Clearing Buffers
        row1.clear();
        row2.clear();

        delay.delay_ms(500_u32);
    }
}

#[interrupt]
fn TIM1_UP_TIM16() {
    cortex_m::interrupt::free(|cs| {
        // Moving out comp
        let mut comp = G_COMP.borrow(cs).replace(None).unwrap();

        // Handle Callback
        comp.handle_callback();
        comp.reset_timer();

        // Moving comp back
        *G_COMP.borrow(cs).borrow_mut() = Some(comp);
    });
}
