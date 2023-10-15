fn main() {
    let description = graphiti::describe! {
        nodes: {
            device: "device",
            safety: "safety",
            controller: "controller",
            power: "power",
            control: "control",
            io: "io"
        },
        edges: {
            "config_standard": {
                device: [safety, controller, power, control, io],
                safety: [controller, power]
            },
            "config_alternate": {
                device: [controller, control, io],
                controller: [power]
            }
        }
    };
    println!("{description:#?}");
}
