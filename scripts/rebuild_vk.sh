cd $(dirname $0)/..
set -ex


make -C webrogue-sdk/libraries/Vulkan-Docs/xml/ install
cp webrogue-sdk/libraries/Vulkan-Docs/xml/vk.xml webrogue-sdk/libraries/mesa/src/vulkan/registry/vk.xml
make -C webrogue-sdk/libraries/Vulkan-Headers/ --file=Makefile.release update-headers
cp webrogue-sdk/libraries/Vulkan-Headers/include/vulkan/*.h webrogue-sdk/libraries/mesa/include/vulkan
cp webrogue-sdk/libraries/Vulkan-Headers/include/vk_video/*.h webrogue-sdk/libraries/mesa/include/vk_video
cp webrogue-sdk/libraries/Vulkan-Headers/include/vulkan/*.h webrogue-sdk/libraries/SDL2/src/video/khronos/vulkan
cp webrogue-sdk/libraries/Vulkan-Headers/include/vulkan/*.h webrogue-sdk/libraries/SDL3/src/video/khronos/vulkan


cd webrogue-sdk/libraries/mesa/src/gfxstream/codegen
sh generate-gfxstream-vulkan.sh ../../../../../../external/gfxstream
cd ../../../../../..
cd webrogue-sdk/libraries/mesa/src/gfxstream/guest/vulkan_enc
rm func_table.cpp \
    goldfish_vk_counting_guest.cpp goldfish_vk_counting_guest.h \
    goldfish_vk_deepcopy_guest.cpp goldfish_vk_deepcopy_guest.h \
    goldfish_vk_extension_structs_guest.cpp goldfish_vk_extension_structs_guest.h \
    goldfish_vk_marshaling_guest.cpp goldfish_vk_marshaling_guest.h \
    goldfish_vk_reserved_marshaling_guest.cpp goldfish_vk_reserved_marshaling_guest.h \
    goldfish_vk_transform_guest.cpp goldfish_vk_transform_guest.h \
    VkEncoder.cpp VkEncoder.h \
    vulkan_gfxstream_structure_type.h \
    vulkan_gfxstream.h
cd ../../../../../../..
