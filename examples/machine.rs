trait MachineModule {
    fn name(&self) -> String;
    fn details(&self) -> String;
}

struct LightModule {
    id: u32,
    brightness: u8,
    color: String,
}

impl MachineModule for LightModule {
    fn name(&self) -> String {
        format!("LightModule_{}", self.id)
    }

    fn details(&self) -> String {
        format!("Brightness: {}, Color: {}", self.brightness, self.color)
    }
}

struct HvacModule {
    id: u32,
    temperature: f32,
    fan_speed: u8,
}

impl MachineModule for HvacModule {
    fn name(&self) -> String {
        format!("HvacModule_{}", self.id)
    }

    fn details(&self) -> String {
        format!(
            "Temperature: {}°C, Fan Speed: {}",
            self.temperature, self.fan_speed
        )
    }
}

macro_rules! machine_description {
    (
        groups: [$($group:ident),*],
        devices: {$($device:ident: [$($modules:expr),* $(,)?]),* $(,)?},
        connections: {$($conn_group:ident: [$($conn_device:ident),* $(,)?]),* $(,)?}
    ) => {{
        (|| -> Result<graphiti::Description, graphiti::Error> {
            let mut builder = graphiti::DescriptionBuilder::new();
            let mut device_mapping: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

            // Adding nodes for groups
            $(
                builder.add_node(stringify!($group).to_string(), ("Group".to_string(),))?;
            )*

            // Adding nodes for devices and maintaining a mapping of device identifier to module names
            $(
                let mut module_names = Vec::new();
                $(
                    let name = $modules.name();
                    module_names.push(name.clone());
                    builder.add_node(name, ($modules.details(),))?;
                )*
                device_mapping.insert(stringify!($device).to_string(), module_names);
            )*

            // Adding edges using the correct node names
            $(
                let source_name = stringify!($conn_group).to_string();
                $(
                    let targets = device_mapping.get(&stringify!($conn_device).to_string()).unwrap();
                    for target_name in targets {
                        builder.add_edge("Contains", &source_name, vec![target_name])?;
                    }
                )*
            )*

            Ok(builder.build())
        })()
    }};
}

fn main() {
    let light_module_1 = LightModule {
        id: 1,
        brightness: 80,
        color: "White".to_string(),
    };
    let hvac_module_1 = HvacModule {
        id: 1,
        temperature: 22.5,
        fan_speed: 3,
    };
    let hvac_module_2 = HvacModule {
        id: 2,
        temperature: 20.0,
        fan_speed: 2,
    };

    let description_result = machine_description! {
        groups: [group_1, group_2],
        devices: {
            device_1: [light_module_1, hvac_module_1],
            device_2: [hvac_module_2]
        },
        connections: {
            group_1: [device_1],
            group_2: [device_1, device_2]
        }
    };

    match description_result {
        Ok(description) => {
            println!("{:#?}", description);

            // Get the dot representation for the "edge1" graph
            if let Some(dot_string) = description.graphs.to_dot("edge1") {
                std::fs::write("edge1.dot", dot_string).expect("Unable to write to file");
                println!("edge1.dot created successfully!");
            } else {
                panic!("Graph 'edge1' not found!");
            }
        }
        Err(error) => {
            eprintln!("Error: {:?}", error);
        }
    }
}
