use caiman_rt::wgpu;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub struct Instance {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Instance {
    fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter_future = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        });
        let adapter = futures::executor::block_on(adapter_future).unwrap();
        // TODO: We should not be using MAPPABLE_PRIMARY_BUFFERS.
        // As WGPU notes, it's a really terrible idea on NUMA systems, and those systems are
        // arguably the ones that would benefit the most from Caiman.
        // In the future we should copy from the GPU buffer to a host-resident buffer, and then
        // map the buffer on the host (if we're on NUMA)
        // TODO: This comment doesn't even belong here, really... it's not an issue with the tests
        let device_desc = wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default() | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
            limits: wgpu::Limits::default(),
        };
        let device_future = adapter.request_device(&device_desc, None);
        let (device, queue) = futures::executor::block_on(device_future).unwrap();
        Self { device, queue }
    }
    pub fn device(&mut self) -> &'_ mut wgpu::Device {
        &mut self.device
    }
    pub fn queue(&mut self) -> &'_ mut wgpu::Queue {
        &mut self.queue
    }
    pub fn create_root_state(&mut self) -> caiman_rt::RootState<'_, '_> {
        caiman_rt::RootState::new(&mut self.device, &mut self.queue)
    }
}

pub static INSTANCE: Lazy<Mutex<Instance>> = Lazy::new(|| Mutex::new(Instance::new()));

/// Convienence macro for asserting test outputs without poisoning the global instance on failure.
#[macro_export]
macro_rules! expect_returned {
    ($expected:literal, $result:ident) => {{
        let returned = $result.returned().map(|x| x.0);
        if (Some($expected) == returned) {
            return Ok(());
        } else if let Some(rv) = returned {
            return Err(format!("expected {}, got {}", $expected, rv));
        } else {
            return Err("couldn't unwrap result".to_string());
        }
    }};
}
