use messages_prost::command as cmd;

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
	Online,
	DeployDrogue,
	DeployMain,
	PowerDown,
	PowerUpCamera,
	PowerDownCamera,
	RadioRateLow,
	RadioRateMedium,
	RadioRateHigh,
	Ping,
}

impl MenuItem {
	pub fn all() -> &'static [MenuItem] {
		use MenuItem::*;
		const ITEMS: &[MenuItem] = &[
			Online,
			DeployDrogue,
			DeployMain,
			PowerDown,
			PowerUpCamera,
			PowerDownCamera,
			RadioRateLow,
			RadioRateMedium,
			RadioRateHigh,
			Ping,
		];
		ITEMS
	}

	pub fn label(&self) -> &'static str {
		match self {
			MenuItem::Online => "Online { online: true }",
			MenuItem::DeployDrogue => "DeployDrogue { val: true }",
			MenuItem::DeployMain => "DeployMain { val: true }",
			MenuItem::PowerDown => "PowerDown { board: <node> }",
			MenuItem::PowerUpCamera => "PowerUpCamera",
			MenuItem::PowerDownCamera => "PowerDownCamera",
			MenuItem::RadioRateLow => "RadioRateChange { rate: LOW }",
			MenuItem::RadioRateMedium => "RadioRateChange { rate: MEDIUM }",
			MenuItem::RadioRateHigh => "RadioRateChange { rate: HIGH }",
			MenuItem::Ping => "Ping { id: <auto> }",
		}
	}
}

pub fn build_command(
	item: MenuItem,
	_origin_node: i32,
	target_node: i32,
	ping_counter: &mut u32,
) -> cmd::Command {
	match item {
		MenuItem::Online => cmd::Command {
			node: target_node,
			data: Some(cmd::command::Data::Online(cmd::Online { online: true })),
		},
		MenuItem::DeployDrogue => cmd::Command {
			node: target_node,
			data: Some(cmd::command::Data::DeployDrogue(cmd::DeployDrogue { val: true })),
		},
		MenuItem::DeployMain => cmd::Command {
			node: target_node,
			data: Some(cmd::command::Data::DeployMain(cmd::DeployMain { val: true })),
		},
		MenuItem::PowerDown => cmd::Command {
			node: target_node,
			data: Some(cmd::command::Data::PowerDown(cmd::PowerDown { board: target_node })),
		},
		MenuItem::PowerUpCamera => cmd::Command {
			node: target_node,
			data: Some(cmd::command::Data::PowerUpCamera(cmd::PowerUpCamera {})),
		},
		MenuItem::PowerDownCamera => cmd::Command {
			node: target_node,
			data: Some(cmd::command::Data::PowerDownCamera(cmd::PowerDownCamera {})),
		},
		MenuItem::RadioRateLow => cmd::Command {
			node: target_node,
			data: Some(cmd::command::Data::RadioRateChange(cmd::RadioRateChange {
				rate: cmd::RadioRate::RateLow as i32,
			})),
		},
		MenuItem::RadioRateMedium => cmd::Command {
			node: target_node,
			data: Some(cmd::command::Data::RadioRateChange(cmd::RadioRateChange {
				rate: cmd::RadioRate::RateMedium as i32,
			})),
		},
		MenuItem::RadioRateHigh => cmd::Command {
			node: target_node,
			data: Some(cmd::command::Data::RadioRateChange(cmd::RadioRateChange {
				rate: cmd::RadioRate::RateHigh as i32,
			})),
		},
		MenuItem::Ping => {
			let id = ping_counter.wrapping_add(1);
			*ping_counter = id;
			cmd::Command {
				node: target_node,
				data: Some(cmd::command::Data::Ping(cmd::Ping { id })),
			}
		}
	}
}
