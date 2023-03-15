//! Frequency Sampling using Analog Digital Converter (ADC).
use cortex_m::peripheral::NVIC;

use stm32l4xx_hal::{
    adc::{Adc, Channel, DmaMode, SampleTime, Sequence},
    device::{Interrupt, ADC1},
    dma::{dma1, RxDma, Transfer, W},
};

/// Measures the frequency of a signal using ADC.
pub struct ADC {
    hadc: Option<Adc<ADC1>>,
    adc_buf: Option<&'static mut [u16; 4096]>,
    transfer: Option<Transfer<W, &'static mut [u16; 4096], RxDma<Adc<ADC1>, dma1::C1>>>,
    hdma: Option<dma1::C1>,
    flag: bool,
    freq_bin: f32,
}

impl ADC {
    /// Creates a new ADC struct.
    ///
    /// The struct will continuously measure the adc value measured by
    /// `hadc` using DMA at channel `hdma`.
    pub fn new<C: Channel<ADC1>>(
        mut hadc: Adc<ADC1>,
        adc_buf: &'static mut [u16; 4096],
        mut pin: C,
        hdma: dma1::C1,
        sampling_time: SampleTime,
        freq_bin: f32,
    ) -> Self {
        hadc.configure_sequence(&mut pin, Sequence::One, sampling_time);

        // Setting continous conversion mode
        unsafe {
            let ptr = &*ADC1::PTR;
            ptr.cfgr.write(|w| w.cont().set_bit());
        }

        unsafe { NVIC::unmask(Interrupt::DMA1_CH1) };

        Self {
            hadc: Some(hadc),
            adc_buf: Some(adc_buf),
            transfer: None,
            hdma: Some(hdma),
            flag: false,
            freq_bin,
        }
    }

    /// Starts measuring the frequency.
    pub fn start(&mut self) {
        if self.hadc.is_some() && self.hdma.is_some() && self.adc_buf.is_some() {
            // Gettings values
            let adc = self.hadc.take().unwrap();
            let dma = self.hdma.take().unwrap();
            let buf = self.adc_buf.take().unwrap();

            // Starting DMA
            let transfer = Transfer::from_adc(adc, dma, buf, DmaMode::Oneshot, true);
            self.transfer = Some(transfer);
        }
    }

    /// Stops measuring the frequency.
    pub fn stop(&mut self) {
        self.flag = false;
        if let Some(transfer_val) = self.transfer.take() {
            // Getting back values from DMA
            let (buf, rx_dma) = transfer_val.wait();
            let (adc, mut dma) = rx_dma.split();

            // Stopping DMA
            dma.stop();

            // Putting back values
            self.hadc = Some(adc);
            self.hdma = Some(dma);
            self.adc_buf = Some(buf);
        }
    }

    /// Function to be called when the DMA callback interrupt is called.
    pub fn handle_callback(&mut self) {
        self.stop();
        self.flag = true;
    }

    /// Calculates the frequency measured by the ADC.
    pub fn calculate_frequency(&mut self, restart: bool) -> f32 {
        // Doing FFT
        if self.flag == true {
            self.flag = false;
            // Converting buffer to f32
            let buf = self.adc_buf.take().unwrap();
            let mut samples: [f32; 4096] = buf.map(|v| v as f32);
            self.adc_buf = Some(buf);

            // Doing FFT
            let spectrum = microfft::real::rfft_4096(&mut samples);

            // since the real-valued coefficient at the Nyquist frequency is packed into the
            // imaginary part of the DC bin, it must be cleared before computing the amplitudes
            spectrum[0].im = 0.0;

            // Getting frequency with highest amplitude
            let amplitudes = spectrum.map(|c| c.norm_sqr() as f32);

            // Restarting ADC DMA
            if restart == true {
                self.start();
            }
            let max = amplitudes
                .into_iter()
                .enumerate()
                .reduce(|acc, e| {
                    if acc.1 == f32::max(e.1, acc.1) {
                        acc
                    } else {
                        e
                    }
                })
                .unwrap();
            self.freq_bin * max.0 as f32
        } else {
            f32::NAN
        }
    }
}
