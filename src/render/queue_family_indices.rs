use ash::{khr, vk};

pub struct QueueFamilyIndices {
    pub main: u32,
    pub present: u32,
}

impl QueueFamilyIndices {
    pub fn new(
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        instance: &ash::Instance,
        surface_instance: &khr::surface::Instance,
    ) -> Option<Self> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let mut main_family_index = None;
        let mut present_family_index = None;

        for (index, family) in queue_families
            .iter()
            .enumerate()
            .map(|(i, f)| (i as u32, f))
        {
            if main_family_index.is_none()
                && family.queue_flags.contains(
                    vk::QueueFlags::GRAPHICS | vk::QueueFlags::TRANSFER | vk::QueueFlags::COMPUTE,
                )
            {
                main_family_index = Some(index);
                continue;
            }

            if unsafe {
                surface_instance.get_physical_device_surface_support(
                    physical_device,
                    index,
                    surface,
                )
            }
            .unwrap_or(false)
            {
                present_family_index = Some(index);
            }
        }

        if present_family_index.is_none() {
            present_family_index = main_family_index;
        }

        Some(Self {
            main: main_family_index?,
            present: present_family_index?,
        })
    }
}
