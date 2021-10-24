#![no_std]
#![no_main]
#[allow(non_upper_case_globals)]
#[allow(unused_assignments)]

use longan_nano::hal::{pac, eclic::*, prelude::*, time::*, timer::*};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle};
use riscv_rt::entry;
use longan_nano::{lcd, lcd_pins};

use panic_halt as _;

static mut TIMER : Option<Timer<longan_nano::hal::pac::TIMER1>> = None;
static mut LCD_STATE : u8 = 1;


#[no_mangle]
#[allow(non_snake_case)]
fn TIMER1(){
	unsafe {
		riscv::interrupt::disable();
		TIMER.as_mut().unwrap().clear_update_interrupt_flag();
		LCD_STATE = if LCD_STATE == 1 {0} else {1};
		riscv::interrupt::enable();
	}
}


#[entry]
fn main() -> ! {

	let dp = pac::Peripherals::take().unwrap();

	// clock config
	let mut rcu = dp
		.RCU
		.configure()
		.ext_hf_clock(8.mhz())
		.sysclk(108.mhz())
		.freeze();	
	
	let mut afio = dp.AFIO.constrain(&mut rcu);
	let gpioa = dp.GPIOA.split(&mut rcu);
	let gpiob = dp.GPIOB.split(&mut rcu);
	let lcd_pins = lcd_pins!(gpioa, gpiob);
	let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
	let (width, height) = (lcd.size().width as i32, lcd.size().height as i32);

	Rectangle::new(Point::new(0, 0), Size::new(width as u32, height as u32))
		.into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
		.draw(&mut lcd)
		.unwrap();

	longan_nano::hal::pac::ECLIC::reset();
	longan_nano::hal::pac::ECLIC::set_level_priority_bits(LevelPriorityBits::L3P1);
	longan_nano::hal::pac::ECLIC::set_threshold_level(Level::L0);
	longan_nano::hal::pac::ECLIC::setup(pac::Interrupt::TIMER1, TriggerType::Level, Level::L1, Priority::P1);


	unsafe{
		let mut timer = Timer::timer1(dp.TIMER1, 1.hz(), &mut rcu);
		timer.listen(Event::Update);
		TIMER = Some(timer);
		longan_nano::hal::pac::ECLIC::unmask(pac::Interrupt::TIMER1);
		riscv::interrupt::enable();
	}
	loop {
		let new_color;
		let first = Rgb565::new(255, 0, 0);
		let second = Rgb565::new(0, 255, 0);
		unsafe{
			new_color = if LCD_STATE == 1 {first} else {second};
		}

		Rectangle::new(Point::new(0,0), Size::new(width as u32, height as u32))
		.into_styled(PrimitiveStyle::with_fill(new_color))
		.draw(&mut lcd)
		.unwrap();	

		unsafe{riscv::asm::wfi();}
	}

}
