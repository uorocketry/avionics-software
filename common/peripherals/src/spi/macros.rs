#[macro_export]
macro_rules! with_cs {
	($self:expr, $block:expr) => {{
		// Set CS low to start transaction
		$self.cs.set_low();
		// Small delay might be needed depending on SPI speed and device, though often not.
		// $self.delay.delay_us(1);
		let result = $block;
		// Set CS high to end transaction
		$self.cs.set_high();
		result
	}};
}
