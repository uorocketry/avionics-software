// import { graphql } from 'gql.tada';

// export const InsertQuaternionDocument = graphql(`
// 	mutation InsertQuaternionMutation(
// 		$time_stamp: Int = 10
// 		$time_stamp1: Int = 10
// 		$w: Float = 1.5
// 		$x: Float = 1.5
// 		$y: Float = 1.5
// 		$z: Float = 1.5
// 	) {
// 		insert_rocket_message(
// 			objects: {
// 				rocket_sensor_message: {
// 					data: {
// 						rocket_sensor_quat: {
// 							data: {
// 								status: 10
// 								quat_w: $w
// 								quat_x: $x
// 								quat_y: $y
// 								quat_z: $z
// 								euler_std_dev_x: 0
// 								euler_std_dev_y: 0
// 								euler_std_dev_z: 0
// 								time_stamp: $time_stamp1
// 							}
// 						}
// 						component_id: 0
// 					}
// 				}
// 				time_stamp: $time_stamp
// 				sender: "phone"
// 				message_type: "sensor"
// 			}
// 		) {
// 			affected_rows
// 		}
// 	}
// `);
