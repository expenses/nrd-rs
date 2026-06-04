#pragma once
#include <cstdint>
#include <cstdio>
#include <cstring>

#include "NRD.h"
#include "NRI.h"
#include "Extensions/NRIHelper.h"
#include "Extensions/NRIRayTracing.h"
#include "Extensions/NRIWrapperVK.h"
#include "NRDSettings.h"

// autocxx uses clang; MSVC compiles wrapper.cpp.
// NRDIntegration.h includes <map> and <vector>, whose MSVC-internal types
// (e.g. _Mytree) leak into autocxx's generated Rust code when parsed by
// clang.  We provide clang with only what it needs to see.
#ifdef __clang__
namespace nrd {

struct Integration;
struct ResourceSnapshot;

struct IntegrationCreationDesc {
    char name[64];
    float residencyPriority;
    uint16_t resourceWidth;
    uint16_t resourceHeight;
    uint8_t queuedFrameNum;
    bool enableWholeLifetimeDescriptorCaching;
    bool autoWaitForIdle;
    bool demoteFloat32to16;
    bool promoteFloat16to32;
};

} // namespace nrd
#else
#include "NRDIntegration.h"
#endif

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
);

void nrdDestroyDevice(nri::Device* device);

void nrdResourceSnapshotSetResource(
    nrd::ResourceSnapshot* snapshot,
    nrd::ResourceType resourceType,
    nri::Texture* texture,
    nri::AccessBits stateAccess,
    nri::Layout stateLayout,
    nri::StageBits stateStages
);

bool nrdResourceSnapshotGetFinalState(
    const nrd::ResourceSnapshot* snapshot,
    nrd::ResourceType resourceType,
    nri::AccessBits* outAccess,
    nri::Layout* outLayout,
    nri::StageBits* outStages
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

nrd::Integration* nrdCreateIntegration();
void nrdDestroyIntegration(nrd::Integration* integration);

nrd::ResourceSnapshot* nrdCreateResourceSnapshot();
void nrdDestroyResourceSnapshot(nrd::ResourceSnapshot* snapshot);

void nrdIntegrationNewFrame(nrd::Integration* integration);

void nrdIntegrationDenoise(
    nrd::Integration* integration,
    const uint32_t* denoisers,
    uint32_t denoisersNum,
    nri::CommandBuffer* commandBuffer,
    nrd::ResourceSnapshot* resourceSnapshot
);


double nrdIntegrationGetTotalMemoryUsageInMb(const nrd::Integration* integration);
double nrdIntegrationGetPersistentMemoryUsageInMb(const nrd::Integration* integration);
double nrdIntegrationGetAliasableMemoryUsageInMb(const nrd::Integration* integration);

nrd::CommonSettings nrdDefaultCommonSettings();
nrd::RelaxSettings nrdDefaultRelaxSettings();
nrd::ReblurSettings nrdDefaultReblurSettings();
nrd::SigmaSettings nrdDefaultSigmaSettings();
nrd::IntegrationCreationDesc nrdDefaultIntegrationCreationDesc();

