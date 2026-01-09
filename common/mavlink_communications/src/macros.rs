#[macro_export]
/// Initializes the global pool for publishers.
/// # Parameters
/// - `$num_of_tasks`: The maximum number of concurrent publisher tasks. TODO: Check what happens if user enters too little of a task pool size (lockup, hardfault, etc)
/// # Functions
/// - `start_publishers`: Function used to start the publisher tasks
macro_rules! initialize_publisher_pool {
    ($num_of_tasks:literal) => {
        pub fn start_publishers(
            publishers: &'static mut [&'static utils::AsyncMutex<dyn mavlink_communications_traits::publish_subscribe_tools::publisher::PeriodicPublisher>],
            task_scheduler: &Spawner,
            mavlink: &'static utils::AsyncMutex<mavlink_service::service::MavlinkServiceTx>,
        ) {
            for i in publishers.iter() {
                task_scheduler.must_spawn(__start_publisher(i, mavlink));
            }
        }

        #[task(pool_size = $num_of_tasks)]
        async fn __start_publisher(
            publisher: &'static utils::AsyncMutex<dyn mavlink_communications_traits::publish_subscribe_tools::publisher::PeriodicPublisher>,
            mavlink: &'static utils::AsyncMutex<mavlink_service::service::MavlinkServiceTx>,
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
                embassy_time::Timer::after(delay).await;
            }
        }
    };
}

#[macro_export]

/// Initializes the global pool for subscribers.
/// # Functions
/// - `start_subscribers`: Function used to start the subscriber tasks
macro_rules! initialize_subscriber_pool {
    () => {
        pub fn start_subscribers(
            subscribers: &'static mut [&'static utils::AsyncMutex<
                dyn mavlink_communications_traits::publish_subscribe_tools::subscriber::Subscriber,
            >],
            task_scheduler: &Spawner,
            mavlink: &'static utils::AsyncMutex<MavlinkServiceRx>,
            delay: embassy_time::Duration,
        ) {
            task_scheduler.spawn(__start_subscribe_tasks(subscribers, mavlink, delay));
        }

        #[task]
        async fn __start_subscribe_tasks(
            subscribers: &'static mut [&'static utils::AsyncMutex<
                dyn mavlink_communications_traits::publish_subscribe_tools::subscriber::Subscriber,
            >],
            mavlink: &'static utils::AsyncMutex<MavlinkServiceRx>,
            delay: embassy_time::Duration,
        ) {
            loop {
                embassy_time::Timer::after(delay).await;

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
