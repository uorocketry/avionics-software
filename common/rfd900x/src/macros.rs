#[macro_export]
macro_rules! rfd_ati {
	($config_block:block) => {
        // Code to enter ati mode
		self.write_all(&[b'+', b'+', b'+']).await;

		// Must wait at least 1 second after entering ati mode
		Timer::after(Duration::from_secs(1)).await;

        $config_block

        // Save changes
        self.write_all(&[b'A', b'T', b'&', b'W', b'\r',  b'\n']).await;
        		Timer::after(Duration::from_millis(100)).await;


        // Reboot
        self.write_all(&[b'A', b'T', b'Z', b'\r',  b'\n']).await;
        		Timer::after(Duration::from_millis(100)).await;

	};

	($rfd: ident, $config_block:block) => {
        // Code to enter ati mode
		$rfd.uart_service.write_all(&[b'+', b'+', b'+']).await;

		// Must wait at least 1 second after entering ati mode
		Timer::after(Duration::from_secs(1)).await;

        $config_block

        // Save changes
        $rfd.uart_service.write_all(&[b'A', b'T', b'&', b'W', b'\r',  b'\n']).await;
		Timer::after(Duration::from_millis(100)).await;

        // Reboot
        $rfd.uart_service.write_all(&[b'A', b'T', b'Z', b'\r',  b'\n']).await;
		Timer::after(Duration::from_millis(100)).await;

	};
}
