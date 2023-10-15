fn main() {
    let machine_description = graphiti::describe! {
        nodes: {
            // Devices
            device_7: "device_7",
            device_1: "device_1",
            device_2: "device_2",
            device_3: "device_3",

            // Modules for device_7
            led_module_7_0: "led_module_7_0",
            sensor_module_7_0: "sensor_module_7_0",
            discovery_module_7_0: "discovery_module_7_0",

            // Sections for led_module_7_0
            id_section_led_7_0: "id_section_led_7_0",
            zones_section_led_7_0: "zones_section_led_7_0",

            // Modules for device_1
            motion_module_1_0: "motion_module_1_0",

            // Sections for motion_module_1_0
            id_section_motion_1_0: "id_section_motion_1_0",
            positions_section_motion_1_0: "positions_section_motion_1_0",
        },
        edges: {
            "has_module": {
                device_7: [led_module_7_0, sensor_module_7_0, discovery_module_7_0],
                device_1: [motion_module_1_0],
            },
            "has_section": {
                led_module_7_0: [id_section_led_7_0, zones_section_led_7_0],
                motion_module_1_0: [id_section_motion_1_0, positions_section_motion_1_0],
            }
        }
    };

    println!("{machine_description:#?}");
}
