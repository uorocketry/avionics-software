#[macro_export]
/// Initializes the global pool for publishers.
/// # Parameters
/// - `$num_of_tasks`: The maximum number of concurrent publisher tasks. TODO: Check what happens if user enters too little of a task pool size (lockup, hardfault, etc)
/// - `$transceiver_type`: The transceiver used for MAVLink (needed to get around embassy's generic refusal for tasks).
/// # Functions
/// - `start_publishers`: Function used to start the publisher tasks
macro_rules! initialize_task_pool {
	($num_of_tasks:literal, $transceiver_type:ty) => {
		pub fn start_publishers(
			publishers: &'static mut [&'static AsyncMutex<dyn PeriodicPublisher>],
			task_scheduler: &Spawner,
			mavlink: &'static AsyncMutex<MavlinkService<$transceiver_type>>,
		) {
			for i in publishers.iter() {
				task_scheduler.must_spawn(_start_publisher(i, mavlink));
			}
		}

		#[task(pool_size = $num_of_tasks)]
		async fn _start_publisher(
			publisher: &'static AsyncMutex<dyn PeriodicPublisher>,
			mavlink_mutex: &'static AsyncMutex<MavlinkService<$transceiver_type>>,
		) {
			loop {
				let mut delay;
				{
					let mut mavlink = mavlink_mutex.lock().await;
					// This should be encapuslated by the publisher struct
					// Struct should hold a reference to the mavlink service (the IO service)
					// !!! ABOVE IS NO LONGER TRUE, PRIOR LOGIC IS NOW ENCAPSULATED ^^^^ !!!
					let mut publisher = publisher.lock().await;
					let sequence = mavlink.get_internal_sequence();
					delay = publisher.get_delay().clone();
					publisher.publish(&mut mavlink.write_buffer, sequence);
					// warn!("A publisher published!");
				}
				Timer::after(delay).await;
			}
		}
	};
}
