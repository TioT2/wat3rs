use ash::{khr, ext, vk, Entry};

use std::{borrow::Cow, ffi::CString};

use super::queue_family_indices::QueueFamilyIndices;

pub struct Kernel {
    pub instance: ash::Instance,
    pub instance_ext_surface: khr::surface::Instance,
    pub instance_ext_debug: ext::debug_utils::Instance,

    pub surface: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,

    pub device: ash::Device,
    pub device_ext_swapchain: khr::swapchain::Device,

    pub debug_messenger: vk::DebugUtilsMessengerEXT,

    pub queue_family_indices: QueueFamilyIndices,
}

#[derive(Clone, Debug, Default)]
pub enum KernelCreateError {
    VulkanError(vk::Result),
    AshLoadingError,
    NoSuitablePhysicalDevices,

    #[default]
    Unknown,
}

impl std::fmt::Display for KernelCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VulkanError(vk_err) => f.write_fmt(format_args!("vulkan error: {vk_err}")),
            Self::AshLoadingError => f.write_str("ash instance functions loading error"),
            Self::NoSuitablePhysicalDevices => f.write_str("no suiting physical devices found"),
            Self::Unknown => f.write_str("unknown error"),
        }
    }
}

impl From<vk::Result> for KernelCreateError {
    fn from(value: vk::Result) -> Self {
        Self::VulkanError(value)
    }
}

extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    fn fold_flags<F>(input: impl Iterator<Item = (F, &'static str)>, test_fn: impl Fn(&F) -> bool) -> String {
        input
            .filter_map(|(f, name)| if test_fn(&f) { Some(name) } else { None })
            .fold(String::new(), |mut accumulator, value| {
                if !accumulator.is_empty() {
                    accumulator.push_str(" | ");
                }
                accumulator.push_str(value);
                accumulator
            })
    }

    println!(
        "\
        VULKAN VALIDATION MESSAGE\n\
            SEVERITY: {}\n\
            TYPE: {}\n\
            MESSAGE: {}\n\
        ",
        fold_flags([
            (vk::DebugUtilsMessageSeverityFlagsEXT::ERROR, "ERROR"),
            (vk::DebugUtilsMessageSeverityFlagsEXT::INFO, "INFO"),
            (vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE, "VERBOSE"),
            (vk::DebugUtilsMessageSeverityFlagsEXT::WARNING, "WARNING"),
        ].into_iter(), |f| message_severity.contains(*f)),
        fold_flags([
            (vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING, "DEVICE_ADDRESS_BINDING"),
            (vk::DebugUtilsMessageTypeFlagsEXT::GENERAL,                "GENERAL"),
            (vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,            "PERFORMANCE"),
            (vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,             "VALIDATION"),
        ].into_iter(), |f| message_types.contains(*f)),
        if let Some(r) = unsafe { p_callback_data.as_ref() } {
            unsafe { core::ffi::CStr::from_ptr(r.p_message) }
                .to_string_lossy()
        } else {
            Cow::Borrowed("<unknown>")
        }
    );

    return vk::FALSE;
}


impl Kernel {
    pub fn new(
        window_handle: raw_window_handle::RawWindowHandle,
        display_handle: raw_window_handle::RawDisplayHandle,
    ) -> Result<Self, KernelCreateError> {
        let entry = unsafe { Entry::load() }.map_err(|_| KernelCreateError::AshLoadingError)?;

        let mut debug_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::empty()
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            )
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::empty()
                | vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            )
            .pfn_user_callback(Some(vulkan_debug_callback))
            ;

        let instance = unsafe {
            let app_name = CString::new("WAT3RS").unwrap();

            let extension_names = {
                let mut names = vec![
                    ext::debug_utils::NAME.as_ptr()
                ];

                names.extend_from_slice(ash_window::enumerate_required_extensions(display_handle)?);

                names
            };

            let layer_names = {
                let names = vec![
                    c"VK_LAYER_KHRONOS_validation".as_ptr(),
                    c"VK_LAYER_RENDERDOC_Capture".as_ptr(),
                ];
                names
            };

            entry
                .create_instance(
                    &vk::InstanceCreateInfo::default()
                        .application_info(
                            &vk::ApplicationInfo::default()
                                .api_version(vk::API_VERSION_1_0)
                                .application_name(app_name.as_c_str()),
                        )
                        .enabled_extension_names(&extension_names)
                        .enabled_layer_names(&layer_names)
                        .push_next(&mut debug_messenger_create_info),
                    None,
                )?
        };
        let instance_ext_surface = khr::surface::Instance::new(&entry, &instance);
        let instance_ext_debug = ext::debug_utils::Instance::new(&entry, &instance);

        let debug_messenger = unsafe { instance_ext_debug.create_debug_utils_messenger(&debug_messenger_create_info, None)? };

        let surface = unsafe { ash_window::create_surface(&entry, &instance, display_handle, window_handle, None) }?;

        let (physical_device, queue_family_indices) =
            unsafe { instance.enumerate_physical_devices() }
                ?
                .iter()
                .copied()
                .filter_map(|physical_device| {
                    let queue_family_indices = QueueFamilyIndices::new(
                        physical_device,
                        surface,
                        &instance,
                        &instance_ext_surface,
                    )?;
                    let properties =
                        unsafe { instance.get_physical_device_properties(physical_device) };

                    let rate = properties.limits.max_image_dimension2_d;

                    Some((physical_device, queue_family_indices, rate))
                })
                .fold(
                    Option::<(vk::PhysicalDevice, QueueFamilyIndices, u32)>::None,
                    |best, info| {
                        if best
                            .as_ref()
                            .map(|best_info| best_info.2 < info.2)
                            .unwrap_or(true)
                        {
                            Some(info)
                        } else {
                            best
                        }
                    },
                )
                .map(|(device, queue_family_indices, _)| (device, queue_family_indices))
                .ok_or(KernelCreateError::NoSuitablePhysicalDevices)?;

        let device = {
            let queue_create_infos = if queue_family_indices.main == queue_family_indices.present {
                vec![
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(queue_family_indices.main)
                        .queue_priorities(&[1.0, 0.5])
                ]
            } else {
                vec![
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(queue_family_indices.main)
                        .queue_priorities(&[1.0]),
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(queue_family_indices.present)
                        .queue_priorities(&[0.5]),
                ]
            };

            let enabled_extension_names = vec![khr::swapchain::NAME.as_ptr()];

            unsafe {
                instance.create_device(
                    physical_device,
                    &vk::DeviceCreateInfo::default()
                        .enabled_extension_names(&enabled_extension_names)
                        .queue_create_infos(&queue_create_infos),
                    None,
                )
            }?
        };
        let device_ext_swapchain = khr::swapchain::Device::new(&instance, &device);

        Ok(Kernel {
            device,
            instance,
            physical_device,
            surface,
            instance_ext_surface,
            instance_ext_debug,
            debug_messenger,
            device_ext_swapchain,
            queue_family_indices,
        })
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance_ext_debug.destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance_ext_surface.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
