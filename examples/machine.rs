trait MachineModule {
    fn name(&self) -> String;
    fn details(&self) -> String;
}

struct LightModule {
    id: String,
    brightness: u8,
    color: String,
}

impl MachineModule for LightModule {
    fn name(&self) -> String {
        format!("Light {}", self.id)
    }

    fn details(&self) -> String {
        format!("Brightness: {}, Color: {}", self.brightness, self.color)
    }
}

struct HvacModule {
    id: String,
    temperature: f32,
    fan_speed: u8,
}

impl MachineModule for HvacModule {
    fn name(&self) -> String {
        format!("HVAC {}", self.id)
    }

    fn details(&self) -> String {
        format!(
            "Temperature: {}°C, Fan Speed: {}%",
            self.temperature, self.fan_speed
        )
    }
}

// New additions for config.json representation
trait ConfigModule {
    fn identity(&self) -> (String, String);
    fn sections(&self) -> String;
}

struct ZoneModule {
    owner: String,
    subject: String,
    sections: String,
}

impl ConfigModule for ZoneModule {
    fn identity(&self) -> (String, String) {
        (self.owner.clone(), self.subject.clone())
    }

    fn sections(&self) -> String {
        self.sections.clone()
    }
}

struct LightConfigModule {
    owner: String,
    subject: String,
    zones: Vec<u32>,
}

impl ConfigModule for LightConfigModule {
    fn identity(&self) -> (String, String) {
        (self.owner.clone(), self.subject.clone())
    }

    fn sections(&self) -> String {
        format!("{:?}", self.zones)
    }
}

struct ConfigDescriptionBuilder {
    builder: graphiti::DescriptionBuilder,
    module_mapping: std::collections::HashMap<String, Vec<String>>,
}

impl ConfigDescriptionBuilder {
    fn new() -> Self {
        Self {
            builder: graphiti::DescriptionBuilder::new(),
            module_mapping: std::collections::HashMap::new(),
        }
    }

    fn add_zone(&mut self, zone: &str) -> Result<(), graphiti::Error> {
        self.builder
            .add_node(zone.to_string(), ("Zone".to_string(),))
            .map(|_| ())
    }

    fn add_module<T: ConfigModule>(
        &mut self,
        module_type: &str,
        module_instance: &T,
    ) -> Result<(), graphiti::Error> {
        let (owner, subject) = module_instance.identity();
        let name = format!("{}_{}", owner, subject);
        self.module_mapping
            .entry(module_type.to_string())
            .or_insert(Vec::new())
            .push(name.clone());
        self.builder
            .add_node(name, (module_instance.sections(),))
            .map(|_| ())
    }

    fn add_connection(&mut self, zone: &str, module_type: &str) -> Result<(), graphiti::Error> {
        let source_name = zone.to_string();
        if let Some(targets) = self.module_mapping.get(module_type) {
            for target_name in targets {
                self.builder
                    .add_edge("Contains", &source_name, vec![target_name])?;
            }
        }
        Ok(())
    }

    fn build(self) -> Result<graphiti::Description, graphiti::Error> {
        Ok(self.builder.build())
    }
}

macro_rules! config_description {
    (
        zones: [$($zone:ident),*],
        modules: {$($module_type:ident: [$($module_instance:expr),* $(,)?]),* $(,)?},
        connections: {$($zone_conn:ident: [$($module_conn:ident),* $(,)?]),* $(,)?}
    ) => {{
        (|| -> Result<graphiti::Description, graphiti::Error> {
            let mut config_builder = ConfigDescriptionBuilder::new();

            // Adding nodes for zones
            $(
                config_builder.add_zone(stringify!($zone))?;
            )*

            // Adding nodes for modules
            $(
                $(
                    config_builder.add_module(stringify!($module_type), &$module_instance)?;
                )*
            )*

            // Adding edges using the correct node names
            $(
                $(
                    config_builder.add_connection(stringify!($zone_conn), stringify!($module_conn))?;
                )*
            )*

            config_builder.build()
        })()
    }};
}

fn main() -> Result<(), graphiti::Error> {
    // Example data resembling the structure from config.json
    let zone1 = ZoneModule {
        owner: "owner1".to_string(),
        subject: "subject1".to_string(),
        sections: "some_section_data".to_string(),
    };

    let light_module1 = LightConfigModule {
        owner: "owner1".to_string(),
        subject: "subject1".to_string(),
        zones: vec![1, 2, 3],
    };

    // Using the config_description! macro to build a Description
    let description = config_description! {
        zones: [zoneA, zoneB],
        modules: {
            ZoneModule: [zone1],
            LightConfigModule: [light_module1]
        },
        connections: {
            zoneA: [ZoneModule, LightConfigModule],
            zoneB: [LightConfigModule]
        }
    }?;

    // Print or process the description as needed
    println!("{:?}", description);

    Ok(())
}
