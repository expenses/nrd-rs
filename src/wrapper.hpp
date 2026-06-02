#pragma once
#include <cstdint>
#include <cstdio>
#include <cstring>

#include "NRD.h"
#include "NRI.h"
#define NRI_HELPER_H
#include "Extensions/NRIRayTracing.h"
#include "Extensions/NRIWrapperVK.h"
#include "NRDIntegration.h"
#include "NRDSettings.h"

struct NrdQueueFamilyDesc {
    uint32_t queueNum;
    nri::QueueType queueType;
    uint32_t familyIndex;
};

nri::Device* nrdCreateDeviceVK(
    uint64_t vkInstance,
    uint64_t vkPhysicalDevice,
    uint64_t vkDevice,
    const NrdQueueFamilyDesc* queueFamilies,
    uint32_t queueFamilyNum,
    bool enableNRIValidation,
    uint32_t minorVersion,
    void* deviceExtensions,
    uint32_t deviceExtensionNum
);

void nrdDestroyDevice(nri::Device* device);

void nrdResourceSnapshotSetResource(
    nrd::ResourceSnapshot& snapshot,
    nrd::ResourceType resourceType,
    nri::Texture& texture,
    nri::AccessBits stateAccess,
    nri::Layout stateLayout,
    nri::StageBits stateStages
);

bool nrdResourceSnapshotGetFinalState(
    nrd::ResourceSnapshot& snapshot,
    nrd::ResourceType resourceType,
    uint32_t& outAccess,
    uint32_t& outLayout,
    uint32_t& outStages
);

bool nrdIntegrationSetCommonSettings(
    nrd::Integration* integration,
    const nrd::CommonSettings* commonSettings
);

bool nrdIntegrationSetDenoiserSettings(
    nrd::Integration* integration,
    uint32_t denoiser,
    const void* denoiserSettings
);

bool nrdIntegrationRecreate(
    nrd::Integration* integration,
    const nrd::IntegrationCreationDesc* integrationDesc,
    const nrd::DenoiserDesc* denoisers,
    uint32_t denoisersNum,
    nri::Device* device
);

struct NrdTextureVKDesc {
    uint64_t vkImage;
    int32_t vkFormat;
    int32_t vkImageType;
    uint32_t vkImageUsageFlags;
    uint16_t width;
    uint16_t height;
    uint16_t depth;
    uint16_t mipNum;
    uint16_t layerNum;
    uint8_t sampleNum;
};

struct NrdCommandBufferVKDesc {
    uint64_t vkCommandBuffer;
    nri::QueueType queueType;
};

nri::Texture*
nrdCreateTextureVK(nri::Device* device, const NrdTextureVKDesc* desc);

void nrdDestroyTexture(nri::Device* device, nri::Texture* texture);

nri::CommandBuffer* nrdCreateCommandBufferVK(
    nri::Device* device,
    const NrdCommandBufferVKDesc* desc
);

void nrdDestroyCommandBuffer(
    nri::Device* device,
    nri::CommandBuffer* commandBuffer
);

void nrdConstructIntegration(nrd::Integration* p);

nrd::CommonSettings nrdDefaultCommonSettings();
nrd::RelaxSettings nrdDefaultRelaxSettings();
nrd::IntegrationCreationDesc nrdDefaultIntegrationCreationDesc();
