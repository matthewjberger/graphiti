macro_rules! machine {
    (
        devices: {
            $($device_name:ident: {
                modules: {
                    $($module_name:ident: {
                        sections: [$($section_name:expr),* $(,)*]
                    }),* $(,)*
                }
            }),* $(,)*
        }
    ) => {{
        let mut builder = graphiti::DescriptionBuilder::new();

        // Add device nodes
        $(
            builder.add_node(stringify!($device_name).to_string(), stringify!($device_name).to_string()).unwrap();
        )*

        // Add module nodes and connect them to devices
        $(
            $(
                let module_str = format!("{}_{}", stringify!($device_name), stringify!($module_name));
                builder.add_node(module_str.clone(), module_str.clone()).unwrap();
                builder.add_edge(stringify!($device_name).to_string(), stringify!($device_name).to_string(), vec![module_str]).unwrap();
            )*
        )*

        // Add section nodes and connect them to modules
        $(
            $(
                $(
                    let section_str = format!("{}_{}_{}", stringify!($device_name), stringify!($module_name), stringify!($section_name));
                    builder.add_node(section_str.clone(), section_str.clone()).unwrap();
                    let module_str = format!("{}_{}", stringify!($device_name), stringify!($module_name));
                    builder.add_edge(module_str.clone(), module_str, vec![section_str]).unwrap();
                )*
            )*
        )*

        builder.build()
    }};
}

fn main() {
    let machine_description = machine! {
        devices: {
            device: {
                modules: {
                    light_module: {
                        sections: [identity_section, zones_section]
                    },
                    hvac_module: {
                        sections: []
                    },
                    discovery_module: {
                        sections: []
                    }
                }
            },
            another_device: {
                modules: {
                    conveyance_module: {
                        sections: [identity_section, positions_section]
                    }
                }
            }
        }
    };

    println!("{machine_description:#?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_macro() {
        let machine = machine! {
            devices: {
                device1: {
                    modules: {
                        module_a: {
                            sections: ["section1", "section2"]
                        },
                        module_b: {
                            sections: ["section3", "section4"]
                        }
                    }
                },
                device2: {
                    modules: {
                        module_c: {
                            sections: ["section5"]
                        }
                    }
                }
            }
        };
    }
}
