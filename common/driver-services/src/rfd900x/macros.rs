#[macro_export]
/// Runs code in the Hayes Attention Information query mode
macro_rules! in_rfd_ati_mode {
	($config_block:block) => {
        // Code to enter ati mode
		self.write(&[b'+', b'+', b'+']).await;

		// Must wait at least 1 second to enter ati mode
		Timer::after(Duration::from_secs(1)).await;

        $config_block

        // Save changes
        self.write(&[b'A', b'T', b'&', b'W', b'\r',  b'\n']).await;
        Timer::after(Duration::from_millis(100)).await;


        // Reboot
        self.write(&[b'A', b'T', b'Z', b'\r',  b'\n']).await;
        		Timer::after(Duration::from_millis(100)).await;

	};

	($rfd: ident, $config_block:block) => {
        // Code to enter ati mode
		$rfd.io_service.tx_component.write(&[b'+', b'+', b'+']).await;

		// Must wait at least 1 second to enter ati mode
		Timer::after(Duration::from_secs(1)).await;

        $config_block

        // Save changes
        $rfd.io_service.tx_component.write(&[b'A', b'T', b'&', b'W', b'\r',  b'\n']).await;
		Timer::after(Duration::from_millis(100)).await;

        // Reboot
        $rfd.io_service.tx_component.write(&[b'A', b'T', b'Z', b'\r',  b'\n']).await;
		Timer::after(Duration::from_millis(100)).await;

	};
}
