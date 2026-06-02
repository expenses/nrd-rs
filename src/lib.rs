#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use autocxx::c_void;
use autocxx::prelude::*;

include_cpp! {
    #include "wrapper.hpp"
    safety!(unsafe_ffi)
    generate_pod!("NrdQueueFamilyDesc")
    generate!("nrdCreateDeviceVK")
    generate!("nrdDestroyDevice")
    generate!("nrdResourceSnapshotSetResource")
    generate!("nrdResourceSnapshotGetFinalState")
    generate!("nrd::Integration")
    generate!("nrd::ResourceSnapshot")
    generate_pod!("nrd::DenoiserDesc")
    generate_pod!("nrd::CommonSettings")
    generate_pod!("nrd::IntegrationCreationDesc")
    generate!("nri::QueueType")
    generate_pod!("nrd::ReblurHitDistanceParameters")
    generate_pod!("nrd::ReblurAntilagSettings")
    generate_pod!("nrd::ReblurResponsiveAccumulationSettings")
    generate_pod!("nrd::ReblurConvergenceSettings")
    generate_pod!("nrd::ReblurSettings")
    generate_pod!("nrd::RelaxAntilagSettings")
    generate_pod!("nrd::RelaxSettings")
    generate_pod!("nrd::SigmaSettings")
    generate_pod!("nrd::ReferenceSettings")
    generate!("nrdIntegrationSetCommonSettings")
    generate!("nrdIntegrationSetDenoiserSettings")
    generate!("nrdIntegrationRecreate")
    generate_pod!("NrdTextureVKDesc")
    generate!("nrdCreateTextureVK")
    generate!("nrdDestroyTexture")
    generate_pod!("NrdCommandBufferVKDesc")
    generate!("nrdCreateCommandBufferVK")
    generate!("nrdDestroyCommandBuffer")
    generate!("nrdConstructIntegration")
    generate!("nrdDefaultCommonSettings")
    generate!("nrdDefaultRelaxSettings")
    generate!("nrdDefaultIntegrationCreationDesc")
    name!(generated)
}

pub mod ffi {
    pub use crate::generated::*;
}

pub struct Device {
    inner: *mut ffi::nri::Device,
}

impl Device {
    pub unsafe fn create_vk(
        vk_instance: u64,
        vk_physical_device: u64,
        vk_device: u64,
        queue_families: &[ffi::NrdQueueFamilyDesc],
        enable_nri_validation: bool,
        minor_version: u32,
        device_extensions: &[*const i8],
    ) -> Option<Self> {
        let ptr = ffi::nrdCreateDeviceVK(
            vk_instance,
            vk_physical_device,
            vk_device,
            queue_families.as_ptr(),
            queue_families.len() as u32,
            enable_nri_validation,
            minor_version,
            device_extensions.as_ptr() as *mut autocxx::c_void,
            device_extensions.len() as u32,
        );
        if ptr.is_null() {
            None
        } else {
            Some(Self { inner: ptr })
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::nrdDestroyDevice(self.inner) }
        }
    }
}

pub struct Texture {
    inner: *mut ffi::nri::Texture,
    device: *mut ffi::nri::Device,
}

impl Texture {
    pub unsafe fn create_vk(device: &mut Device, desc: &ffi::NrdTextureVKDesc) -> Option<Self> {
        let inner = ffi::nrdCreateTextureVK(device.inner, desc);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                device: device.inner,
            })
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::nri::Texture {
        self.inner
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::nrdDestroyTexture(self.device, self.inner) }
        }
    }
}

pub struct CommandBuffer {
    inner: *mut ffi::nri::CommandBuffer,
    device: *mut ffi::nri::Device,
}

impl CommandBuffer {
    pub unsafe fn create_vk(
        device: &mut Device,
        desc: &ffi::NrdCommandBufferVKDesc,
    ) -> Option<Self> {
        let inner = ffi::nrdCreateCommandBufferVK(device.inner, desc);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                device: device.inner,
            })
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::nri::CommandBuffer {
        self.inner
    }

    pub fn pin_mut(&mut self) -> std::pin::Pin<&mut ffi::nri::CommandBuffer> {
        unsafe { std::pin::Pin::new_unchecked(&mut *self.inner) }
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::nrdDestroyCommandBuffer(self.device, self.inner) }
        }
    }
}

pub struct ResourceSnapshot {
    inner: cxx::UniquePtr<ffi::nrd::ResourceSnapshot>,
}

impl ResourceSnapshot {
    pub fn new() -> Self {
        Self {
            inner: ffi::nrd::ResourceSnapshot::new1().within_unique_ptr(),
        }
    }

    pub fn set_resource(
        &mut self,
        resource_type: ffi::nrd::ResourceType,
        texture: &mut Texture,
        state_access: ffi::nri::AccessBits,
        state_layout: ffi::nri::Layout,
        state_stages: ffi::nri::StageBits,
    ) {
        unsafe {
            ffi::nrdResourceSnapshotSetResource(
                self.inner.pin_mut(),
                resource_type,
                std::pin::Pin::new_unchecked(&mut *texture.inner),
                state_access,
                state_layout,
                state_stages,
            )
        }
    }

    pub fn get_final_state(
        &mut self,
        resource_type: ffi::nrd::ResourceType,
    ) -> Option<(u32, u32, u32)> {
        let mut out_access: u32 = 0;
        let mut out_layout: u32 = 0;
        let mut out_stages: u32 = 0;
        let success = ffi::nrdResourceSnapshotGetFinalState(
            self.inner.pin_mut(),
            resource_type,
            std::pin::Pin::new(&mut out_access),
            std::pin::Pin::new(&mut out_layout),
            std::pin::Pin::new(&mut out_stages),
        );
        if success {
            Some((out_access, out_layout, out_stages))
        } else {
            None
        }
    }

    pub fn pin_mut(&mut self) -> std::pin::Pin<&mut ffi::nrd::ResourceSnapshot> {
        self.inner.pin_mut()
    }
}

pub struct Integration {
    inner: cxx::UniquePtr<ffi::nrd::Integration>,
}

impl Integration {
    pub fn new(
        integration_desc: &ffi::nrd::IntegrationCreationDesc,
        denoisers: &[ffi::nrd::DenoiserDesc],
        device: &mut Device,
    ) -> Option<Self> {
        let mut inner = ffi::nrd::Integration::new().within_unique_ptr();
        unsafe { ffi::nrdConstructIntegration(&mut *inner) };
        let success = unsafe {
            ffi::nrdIntegrationRecreate(
                &mut *inner,
                integration_desc,
                denoisers.as_ptr(),
                denoisers.len() as u32,
                device.inner,
            )
        };
        if success { Some(Self { inner }) } else { None }
    }

    pub fn new_frame(&mut self) {
        self.inner.pin_mut().NewFrame()
    }

    pub unsafe fn denoise(
        &mut self,
        denoisers: &[u32],
        command_buffer: &mut CommandBuffer,
        resource_snapshot: &mut ResourceSnapshot,
    ) {
        self.inner.pin_mut().Denoise(
            denoisers.as_ptr(),
            denoisers.len() as u32,
            command_buffer.pin_mut(),
            resource_snapshot.pin_mut(),
        );
    }

    pub fn destroy(&mut self) {
        self.inner.pin_mut().Destroy()
    }

    pub fn destroy_cached_descriptors(&mut self) {
        self.inner.pin_mut().DestroyCachedDescriptors()
    }

    pub fn recreate_pipelines(&mut self) -> bool {
        self.inner.pin_mut().RecreatePipelines()
    }

    pub fn total_memory_usage_mb(&self) -> f64 {
        self.inner.GetTotalMemoryUsageInMb()
    }

    pub fn persistent_memory_usage_mb(&self) -> f64 {
        self.inner.GetPersistentMemoryUsageInMb()
    }

    pub fn aliasable_memory_usage_mb(&self) -> f64 {
        self.inner.GetAliasableMemoryUsageInMb()
    }

    pub fn set_common_settings(&mut self, common_settings: &ffi::nrd::CommonSettings) -> bool {
        unsafe { ffi::nrdIntegrationSetCommonSettings(&mut *self.inner, common_settings) }
    }

    pub unsafe fn set_denoiser_settings(
        &mut self,
        denoiser: u32,
        denoiser_settings: *const autocxx::c_void,
    ) -> bool {
        ffi::nrdIntegrationSetDenoiserSettings(&mut *self.inner, denoiser, denoiser_settings)
    }

    pub unsafe fn recreate(
        &mut self,
        integration_desc: &ffi::nrd::IntegrationCreationDesc,
        denoisers: &[ffi::nrd::DenoiserDesc],
        device: &mut Device,
    ) -> bool {
        ffi::nrdIntegrationRecreate(
            &mut *self.inner,
            integration_desc,
            denoisers.as_ptr(),
            denoisers.len() as u32,
            device.inner,
        )
    }
}

impl Default for ffi::nrd::CommonSettings {
    fn default() -> Self {
        ffi::nrdDefaultCommonSettings()
    }
}

impl Default for ffi::nrd::IntegrationCreationDesc {
    fn default() -> Self {
        ffi::nrdDefaultIntegrationCreationDesc()
    }
}
