// TODO: The imports should be absolute paths (ie, utils::AsyncMutex rather than AsyncMutex)

#[macro_export]
/// Initializes the global pool for publishers.
/// # Parameters
/// - `$num_of_tasks`: The maximum number of concurrent publisher tasks. TODO: Check what happens if user enters too little of a task pool size (lockup, hardfault, etc)
/// - `$transceiver_type`: The transceiver used for MAVLink (needed to get around embassy's generic refusal for tasks).
/// # Functions
/// - `start_publishers`: Function used to start the publisher tasks
macro_rules! initialize_publisher_pool {
	($num_of_tasks:literal, $transceiver_type:ty) => {
		pub fn start_publishers(
			publishers: &'static mut [&'static AsyncMutex<dyn PeriodicPublisher>],
			task_scheduler: &Spawner,
			mavlink: &'static AsyncMutex<MavlinkServiceTx>,
		) {
			for i in publishers.iter() {
				task_scheduler.must_spawn(__start_publisher(i, mavlink));
			}
		}

		#[task(pool_size = $num_of_tasks)]
		async fn __start_publisher(
			publisher: &'static AsyncMutex<dyn PeriodicPublisher>,
			mavlink: &'static AsyncMutex<MavlinkServiceTx>,
		) {
			loop {
				let mut delay;
				{
					let mut mavlink = mavlink.lock().await;
					let mut publisher = publisher.lock().await;
					let sequence = mavlink.get_internal_sequence();
					let len = publisher.publish(&mut mavlink.write_buffer, sequence);
					delay = publisher.get_delay().clone();

					if len != 0 {
						mavlink.increment_internal_sequence();
						mavlink.write_internal(len).await;
					}
				}
				Timer::after(delay).await;
			}
		}
	};
}

#[macro_export]

/// Initializes the global pool for subscribers.
/// # Parameters
/// - `$num_of_tasks`: The maximum number of concurrent subscribe tasks. TODO: Check what happens if user enters too little of a task pool size (lockup, hardfault, etc)
/// - `$transceiver_type`: The transceiver used for MAVLink (needed to get around embassy's generic refusal for tasks).
/// # Functions
/// - `start_subscribers`: Function used to start the subscriber tasks
macro_rules! initialize_subscriber_pool {
	($transceiver_type:ty) => {
		pub fn start_subscribers(
			subscribers: &'static mut [&'static AsyncMutex<dyn Subscriber>],
			task_scheduler: &Spawner,
			mavlink: &'static AsyncMutex<MavlinkServiceRx>,
			delay: embassy_time::Duration,
		) {
			task_scheduler.spawn(__start_subscribe_tasks(subscribers, mavlink, delay));
		}

		#[task]
		async fn __start_subscribe_tasks(
			subscribers: &'static mut [&'static AsyncMutex<dyn Subscriber>],
			mavlink: &'static AsyncMutex<MavlinkServiceRx>,
			delay: embassy_time::Duration,
		) {
			loop {
				Timer::after(delay).await;

				if let Ok(frame) = mavlink.lock().await.read_frame().await {
					for i in subscribers.iter() {
						let mut subscriber = i.lock().await;
						subscriber.__update(frame);
					}
				}
			}
		}
	};
}
