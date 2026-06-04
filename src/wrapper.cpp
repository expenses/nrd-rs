#include "wrapper.hpp"

#include <cstdio>

#include "Extensions/NRIHelper.h"
#include "Extensions/NRIRayTracing.h"
#include "Extensions/NRIWrapperVK.h"
#include "NRDIntegration.hpp"
#include "NRI.h"

nri::Device* nrdCreateDeviceVK(
    uint64_t vkInstance,
    uint64_t vkPhysicalDevice,
    uint64_t vkDevice,
    const nri::QueueFamilyVKDesc* queueFamilies,
    uint32_t queueFamilyNum,
    bool enableNRIValidation,
    uint32_t minorVersion,
    void* deviceExtensions,
    uint32_t deviceExtensionNum,
    nri::VKBindingOffsets bindingOffsets
) {
    nri::Device* device = nullptr;

    nri::DeviceCreationVKDesc desc = {};
    desc.vkInstance = reinterpret_cast<void*>(vkInstance);
    desc.vkPhysicalDevice = reinterpret_cast<void*>(vkPhysicalDevice);
    desc.vkDevice = reinterpret_cast<void*>(vkDevice);
    desc.vkBindingOffsets = bindingOffsets;
    desc.queueFamilyNum = queueFamilyNum;
    desc.queueFamilies = queueFamilies;
    desc.minorVersion = minorVersion;
    desc.enableNRIValidation = enableNRIValidation;
    desc.vkExtensions.deviceExtensions = (const char* const*)deviceExtensions;
    desc.vkExtensions.deviceExtensionNum = deviceExtensionNum;

    nri::Result result = nriCreateDeviceFromVKDevice(desc, device);
    if (result != nri::Result::SUCCESS)
        return nullptr;
    return device;
}

void nrdDestroyDevice(nri::Device* device) {
    if (device)
        nriDestroyDevice(device);
}

void nrdResourceSnapshotSetResource(
    nrd::ResourceSnapshot* snapshot,
    nrd::ResourceType resourceType,
    nri::Texture* texture,
    nri::AccessBits stateAccess,
    nri::Layout stateLayout,
    nri::StageBits stateStages
) {
    nrd::Resource resource = {};
    resource.nri.texture = texture;
    resource.state.access = stateAccess;
    resource.state.layout = stateLayout;
    resource.state.stages = stateStages;
    snapshot->SetResource(resourceType, resource);
}

bool nrdResourceSnapshotGetFinalState(
    const nrd::ResourceSnapshot* snapshot,
    nrd::ResourceType resourceType,
    nri::AccessBits* outAccess,
    nri::Layout* outLayout,
    nri::StageBits* outStages
) {
    nrd::Resource* res = snapshot->slots[(size_t)resourceType];
    if (!res)
        return false;
    *outAccess = res->state.access;
    *outLayout = res->state.layout;
    *outStages = res->state.stages;
    return true;
}

bool nrdIntegrationSetCommonSettings(
    nrd::Integration* integration,
    const nrd::CommonSettings* commonSettings
) {
    return integration->SetCommonSettings(*commonSettings)
        == nrd::Result::SUCCESS;
}

bool nrdIntegrationSetDenoiserSettings(
    nrd::Integration* integration,
    uint32_t denoiser,
    const void* denoiserSettings
) {
    return integration->SetDenoiserSettings(denoiser, denoiserSettings)
        == nrd::Result::SUCCESS;
}

bool nrdIntegrationRecreate(
    nrd::Integration* integration,
    const nrd::IntegrationCreationDesc* integrationDesc,
    const nrd::DenoiserDesc* denoisers,
    uint32_t denoisersNum,
    nri::Device* device
) {
    nrd::InstanceCreationDesc instanceDesc = {};
    instanceDesc.denoisers = denoisers;
    instanceDesc.denoisersNum = denoisersNum;

    return integration->Recreate(*integrationDesc, instanceDesc, device)
        == nrd::Result::SUCCESS;
}

nri::Texture*
nrdCreateTextureVK(nri::Device* device, const NrdTextureVKDesc* desc) {
    nri::WrapperVKInterface wrapperVK = {};
    nri::Result result = nri::nriGetInterface(
        *device,
        NRI_INTERFACE(nri::WrapperVKInterface),
        &wrapperVK
    );
    if (result != nri::Result::SUCCESS)
        return nullptr;

    nri::TextureVKDesc nriDesc = {};
    nriDesc.vkImage = (VKNonDispatchableHandle)desc->vkImage;
    nriDesc.vkFormat = (VKEnum)desc->vkFormat;
    nriDesc.vkImageType = (VKEnum)desc->vkImageType;
    nriDesc.vkImageUsageFlags = (VKFlags)desc->vkImageUsageFlags;
    nriDesc.width = desc->width;
    nriDesc.height = desc->height;
    nriDesc.depth = desc->depth;
    nriDesc.mipNum = desc->mipNum;
    nriDesc.layerNum = desc->layerNum;
    nriDesc.sampleNum = desc->sampleNum;

    nri::Texture* texture = nullptr;
    result = wrapperVK.CreateTextureVK(*device, nriDesc, texture);
    return result == nri::Result::SUCCESS ? texture : nullptr;
}

void nrdDestroyTexture(nri::Device* device, nri::Texture* texture) {
    if (!texture)
        return;

    nri::CoreInterface core = {};
    if (nri::nriGetInterface(*device, NRI_INTERFACE(nri::CoreInterface), &core)
        == nri::Result::SUCCESS)
        core.DestroyTexture(texture);
}

nri::CommandBuffer* nrdCreateCommandBufferVK(
    nri::Device* device,
    const NrdCommandBufferVKDesc* desc
) {
    nri::WrapperVKInterface wrapperVK = {};
    nri::Result result = nri::nriGetInterface(
        *device,
        NRI_INTERFACE(nri::WrapperVKInterface),
        &wrapperVK
    );
    if (result != nri::Result::SUCCESS)
        return nullptr;

    nri::CommandBufferVKDesc nriDesc = {};
    nriDesc.vkCommandBuffer = (VKHandle)desc->vkCommandBuffer;
    nriDesc.queueType = desc->queueType;

    nri::CommandBuffer* commandBuffer = nullptr;
    result = wrapperVK.CreateCommandBufferVK(*device, nriDesc, commandBuffer);
    return result == nri::Result::SUCCESS ? commandBuffer : nullptr;
}

void nrdDestroyCommandBuffer(
    nri::Device* device,
    nri::CommandBuffer* commandBuffer
) {
    if (!commandBuffer)
        return;

    nri::CoreInterface core = {};
    if (nri::nriGetInterface(*device, NRI_INTERFACE(nri::CoreInterface), &core)
        == nri::Result::SUCCESS)
        core.DestroyCommandBuffer(commandBuffer);
}

nrd::CommonSettings nrdDefaultCommonSettings() {
    return {};
}

nrd::RelaxSettings nrdDefaultRelaxSettings() {
    return {};
}

nrd::ReblurSettings nrdDefaultReblurSettings() {
    return {};
}

nrd::SigmaSettings nrdDefaultSigmaSettings() {
    return {};
}

nrd::Integration* nrdCreateIntegration() {
    return new nrd::Integration();
}

void nrdDestroyIntegration(nrd::Integration* integration) {
    delete integration;
}

nrd::ResourceSnapshot* nrdCreateResourceSnapshot() {
    return new nrd::ResourceSnapshot();
}

void nrdDestroyResourceSnapshot(nrd::ResourceSnapshot* snapshot) {
    delete snapshot;
}

void nrdIntegrationNewFrame(nrd::Integration* integration) {
    integration->NewFrame();
}

void nrdIntegrationDenoise(
    nrd::Integration* integration,
    const uint32_t* denoisers,
    uint32_t denoisersNum,
    nri::CommandBuffer* commandBuffer,
    nrd::ResourceSnapshot* resourceSnapshot
) {
    integration->Denoise(denoisers, denoisersNum, *commandBuffer, *resourceSnapshot);
}

double nrdIntegrationGetTotalMemoryUsageInMb(const nrd::Integration* integration) {
    return integration->GetTotalMemoryUsageInMb();
}

double nrdIntegrationGetPersistentMemoryUsageInMb(const nrd::Integration* integration) {
    return integration->GetPersistentMemoryUsageInMb();
}

double nrdIntegrationGetAliasableMemoryUsageInMb(const nrd::Integration* integration) {
    return integration->GetAliasableMemoryUsageInMb();
}

nrd::IntegrationCreationDesc nrdDefaultIntegrationCreationDesc() {
    return {};
}

