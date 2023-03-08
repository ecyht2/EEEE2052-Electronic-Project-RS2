use stm32l4xx_hal::{
    comp::Comp,
    device::TIM16,
    timer::{Event, Timer},
};

#[allow(unused)]
pub struct Comparator {
    pub hcomp: Comp,
    htim: Timer<TIM16>,
    timer_frequency: f32,
    current_comp_val: u8,
    ticks: u64,
    clock_ticks: u64,
}

impl Comparator {
    pub fn new(hcomp: Comp, htim: Timer<TIM16>, timer_frequency: f32) -> Self {
        Self {
            hcomp,
            htim,
            timer_frequency,
            current_comp_val: 0,
            ticks: 0,
            clock_ticks: 0,
        }
    }

    pub fn start(&mut self) {
        self.hcomp.start();
        self.htim.listen(Event::TimeOut);
        self.current_comp_val = self.hcomp.get_output_level() as u8;
    }

    pub fn stop(&mut self) {
        self.hcomp.stop();
        self.htim.unlisten(Event::TimeOut);
    }

    pub fn reset_timer(&mut self) {
        self.htim.clear_interrupt(Event::TimeOut);
    }

    pub fn handle_callback(&mut self) {
        if self.clock_ticks > 65535 {
            self.ticks = 0;
            self.clock_ticks = 0;
        }

        let current_comp_val = self.hcomp.get_output_level() as u8;
        if self.current_comp_val != current_comp_val {
            self.ticks += 1;
            self.current_comp_val = current_comp_val;
        }
        self.clock_ticks += 1;
    }

    pub fn calculate_frequency(&self) -> f32 {
        let freq = self.timer_frequency * self.ticks as f32 / self.clock_ticks as f32 / 2.0;
        freq
    }
}
