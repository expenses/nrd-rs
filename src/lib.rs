#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]

pub use autocxx::c_void;
use autocxx::prelude::*;

include_cpp! {
    #include "wrapper.hpp"
    safety!(unsafe_ffi)
    generate_pod!("nri::QueueFamilyVKDesc")
    generate_pod!("nri::VKBindingOffsets")
    generate!("nrdCreateDeviceVK")
    generate!("nrdDestroyDevice")
    generate!("nrdResourceSnapshotSetResource")
    generate!("nrdResourceSnapshotGetFinalState")
    generate!("nrd::Integration")
    generate!("nrd::ResourceSnapshot")
    generate!("nrd::Denoiser")
    generate!("nrd::ResourceType")
    generate_pod!("nrd::DenoiserDesc")
    generate_pod!("nrd::CommonSettings")
    generate_pod!("nrd::IntegrationCreationDesc")
    generate!("nri::QueueType")
    generate!("nri::AccessBits")
    generate!("nri::Layout")
    generate!("nri::StageBits")
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
    generate!("nrdDefaultCommonSettings")
    generate!("nrdDefaultRelaxSettings")
    generate!("nrdDefaultReblurSettings")
    generate!("nrdDefaultSigmaSettings")
    generate!("nrdDefaultIntegrationCreationDesc")
    name!(generated)
}

pub mod ffi {
    pub use crate::generated::*;
}

pub use ffi::nrd::{
    CommonSettings, Denoiser, DenoiserDesc, IntegrationCreationDesc, ReblurAntilagSettings,
    ReblurConvergenceSettings, ReblurHitDistanceParameters, ReblurResponsiveAccumulationSettings,
    ReblurSettings, ReferenceSettings, RelaxAntilagSettings, RelaxSettings, ResourceType,
    SigmaSettings,
};
pub use ffi::nri::{AccessBits, Layout, QueueFamilyVKDesc, QueueType, StageBits, VKBindingOffsets};
pub use ffi::{NrdCommandBufferVKDesc, NrdTextureVKDesc};

/// Wraps an `nri::Device`. NRI devices are thread-safe.
///
/// Must outlive all `Texture`, `CommandBuffer`, and `Integration` instances
/// created from it.
pub struct Device {
    inner: *mut ffi::nri::Device,
}

impl Device {
    /// Creates an NRI device wrapping existing Vulkan handles.
    ///
    /// # Safety
    /// - `vk_instance`, `vk_physical_device`, `vk_device` must be valid Vulkan handles.
    /// - `queue_families` must describe valid queue families.
    /// - The returned `Device` must not outlive the Vulkan handles.
    pub unsafe fn create_vk(
        vk_instance: u64,
        vk_physical_device: u64,
        vk_device: u64,
        queue_families: &[ffi::nri::QueueFamilyVKDesc],
        enable_nri_validation: bool,
        minor_version: u32,
        device_extensions: &[*const i8],
        binding_offsets: ffi::nri::VKBindingOffsets,
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
            binding_offsets,
        );
        if ptr.is_null() {
            None
        } else {
            Some(Self { inner: ptr })
        }
    }
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl Drop for Device {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::nrdDestroyDevice(self.inner) }
        }
    }
}

/// An NRI texture wrapping a Vulkan image.
///
/// # Safety
/// Must not outlive the `Device` it was created from.
pub struct Texture {
    inner: *mut ffi::nri::Texture,
    device: *mut ffi::nri::Device,
}

impl Texture {
    pub unsafe fn create_vk(device: &Device, desc: &ffi::NrdTextureVKDesc) -> Option<Self> {
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
}

unsafe impl Send for Texture {}
unsafe impl Sync for Texture {}

impl Drop for Texture {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::nrdDestroyTexture(self.device, self.inner) }
        }
    }
}

/// An NRI command buffer wrapping a Vulkan command buffer.
///
/// # Safety
/// Must not outlive the `Device` it was created from.
pub struct CommandBuffer {
    inner: *mut ffi::nri::CommandBuffer,
    device: *mut ffi::nri::Device,
}

impl CommandBuffer {
    pub unsafe fn create_vk(device: &Device, desc: &ffi::NrdCommandBufferVKDesc) -> Option<Self> {
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
}

unsafe impl Send for CommandBuffer {}
unsafe impl Sync for CommandBuffer {}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::nrdDestroyCommandBuffer(self.device, self.inner) }
        }
    }
}

/// A snapshot of NRD resource state at a given moment.
pub struct ResourceSnapshot {
    inner: cxx::UniquePtr<ffi::nrd::ResourceSnapshot>,
}

impl ResourceSnapshot {
    /// Associates a texture with a resource slot inside the snapshot.
    ///
    /// # Safety
    /// The caller must ensure `texture` outlives all uses of this snapshot.
    pub fn set_resource(
        &mut self,
        resource_type: ffi::nrd::ResourceType,
        texture: &Texture,
        state_access: ffi::nri::AccessBits,
        state_layout: ffi::nri::Layout,
        state_stages: ffi::nri::StageBits,
    ) {
        unsafe {
            ffi::nrdResourceSnapshotSetResource(
                std::pin::Pin::new_unchecked(&mut *self.inner),
                resource_type,
                std::pin::Pin::new_unchecked(&mut *texture.inner),
                state_access,
                state_layout,
                state_stages,
            )
        }
    }

    /// Returns the final resource state after a denoiser pass.
    pub fn get_final_state(
        &self,
        resource_type: ffi::nrd::ResourceType,
    ) -> Option<(ffi::nri::AccessBits, ffi::nri::Layout, ffi::nri::StageBits)> {
        let mut out_access = ffi::nri::AccessBits::NONE;
        let mut out_layout = ffi::nri::Layout::UNDEFINED;
        let mut out_stages = ffi::nri::StageBits::ALL;
        let success = ffi::nrdResourceSnapshotGetFinalState(
            &self.inner,
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
}

impl Default for ResourceSnapshot {
    fn default() -> Self {
        Self {
            inner: ffi::nrd::ResourceSnapshot::new1().within_unique_ptr(),
        }
    }
}

/// The main NRD integration instance.
///
/// Manages denoiser pipelines, resources, and dispatch.
pub struct Integration {
    inner: cxx::UniquePtr<ffi::nrd::Integration>,
}

impl Default for Integration {
    fn default() -> Self {
        Self {
            inner: ffi::nrd::Integration::new().within_unique_ptr(),
        }
    }
}

impl Integration {
    pub fn new_frame(&mut self) {
        self.inner.pin_mut().NewFrame();
    }

    /// Runs denoising for the specified denoisers.
    ///
    /// # Safety
    /// - `command_buffer` must be in a state suitable for recording.
    /// - `resource_snapshot` must contain all resources required by the denoisers.
    pub unsafe fn denoise(
        &mut self,
        denoisers: &[u32],
        command_buffer: &mut CommandBuffer,
        resource_snapshot: &mut ResourceSnapshot,
    ) {
        self.inner.pin_mut().Denoise(
            denoisers.as_ptr(),
            denoisers.len() as u32,
            std::pin::Pin::new_unchecked(&mut *command_buffer.inner),
            std::pin::Pin::new_unchecked(&mut *resource_snapshot.inner),
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

    pub fn set_reblur_settings(
        &mut self,
        denoiser: ffi::nrd::Denoiser,
        settings: &ffi::nrd::ReblurSettings,
    ) -> bool {
        unsafe {
            ffi::nrdIntegrationSetDenoiserSettings(
                &mut *self.inner,
                denoiser as u32,
                settings as *const _ as *const autocxx::c_void,
            )
        }
    }

    pub fn set_relax_settings(
        &mut self,
        denoiser: ffi::nrd::Denoiser,
        settings: &ffi::nrd::RelaxSettings,
    ) -> bool {
        unsafe {
            ffi::nrdIntegrationSetDenoiserSettings(
                &mut *self.inner,
                denoiser as u32,
                settings as *const _ as *const autocxx::c_void,
            )
        }
    }

    pub fn set_sigma_settings(
        &mut self,
        denoiser: ffi::nrd::Denoiser,
        settings: &ffi::nrd::SigmaSettings,
    ) -> bool {
        unsafe {
            ffi::nrdIntegrationSetDenoiserSettings(
                &mut *self.inner,
                denoiser as u32,
                settings as *const _ as *const autocxx::c_void,
            )
        }
    }

    pub fn set_reference_settings(
        &mut self,
        denoiser: ffi::nrd::Denoiser,
        settings: &ffi::nrd::ReferenceSettings,
    ) -> bool {
        unsafe {
            ffi::nrdIntegrationSetDenoiserSettings(
                &mut *self.inner,
                denoiser as u32,
                settings as *const _ as *const autocxx::c_void,
            )
        }
    }

    pub unsafe fn recreate(
        &mut self,
        integration_desc: &ffi::nrd::IntegrationCreationDesc,
        denoisers: &[ffi::nrd::DenoiserDesc],
        device: &Device,
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

impl Default for ffi::nrd::RelaxSettings {
    fn default() -> Self {
        ffi::nrdDefaultRelaxSettings()
    }
}

impl Default for ffi::nrd::ReblurSettings {
    fn default() -> Self {
        ffi::nrdDefaultReblurSettings()
    }
}

impl Default for ffi::nrd::SigmaSettings {
    fn default() -> Self {
        ffi::nrdDefaultSigmaSettings()
    }
}

impl Default for ffi::nrd::IntegrationCreationDesc {
    fn default() -> Self {
        ffi::nrdDefaultIntegrationCreationDesc()
    }
}

impl Default for ffi::nri::VKBindingOffsets {
    fn default() -> Self {
        Self {
            sRegister: 0,
            tRegister: 1,
            bRegister: 2,
            uRegister: 3,
        }
    }
}

#[test]
fn integration_exists() {
    let _ = super::Integration::default();
    let _ = super::ResourceSnapshot::default();
}
