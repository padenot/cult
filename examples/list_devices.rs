
extern crate cult;

fn list_devices(ctx: &cult::Context, dt: cult::DeviceType) {
    let devices = ctx.enumerate_devices(dt)
            .expect(&format!("Cult could not enumerate {:?} devices", dt));
    println!("{:?} devices:", dt);
    for dev in devices {
        println!("  {}:", dev.device_id());
        println!("    devid:          {:p}", dev.devid());
        println!("    name:           {}", dev.friendly_name());
        println!("    group id:       {}", dev.group_id());
        println!("    vendor:         {}", dev.vendor_name());
        println!("    type:           {:?}", dev.device_type());
        println!("    state:          {:?}", dev.state());
        println!("    pref:           {:?}", dev.preferred());
        println!("    format:         {:?}", dev.format());
        println!("    default format: {:?}", dev.default_format());
        println!("    max channels:   {}", dev.max_channels());
        println!("    default rate:   {}", dev.default_rate());
        println!("    max rate:       {}", dev.max_rate());
        println!("    min rate:       {}", dev.min_rate());
        println!("    latency low:    {}", dev.latency_lo());
        println!("    latency high:   {}", dev.latency_hi());
    }
}

fn main() {
    let ctx = cult::Context::new("", None).unwrap();
    list_devices(&ctx, cult::DeviceType::Input);
    list_devices(&ctx, cult::DeviceType::Output);
}
