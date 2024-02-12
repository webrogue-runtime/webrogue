addToLibrary({
    prepareMods: function () {
        Asyncify.handleSleep(wakeUp => {
            var transaction = homepageIndexedDB.transaction("mods", 'readonly');
            var allRecords = transaction.objectStore("mods").getAll();
            allRecords.onsuccess = function () {
                Module.modsToLoad = allRecords.result;
                async function resolveBlobs() {
                    for (const key in Module.modsToLoad) {
                        if (Object.hasOwnProperty.call(Module.modsToLoad, key)) {
                            const modToLoad = Module.modsToLoad[key];
                            modToLoad.data = new Uint8Array(await modToLoad.blob.arrayBuffer());
                        }
                    }
                }
                resolveBlobs().then(wakeUp);
            };
        });
    },
    modsToLoadCount: function () {
        return Module.modsToLoad.length;
    },
    modToLoadNameSize: function () {
        let modName = Module.modsToLoad[0].modName;
        let encodedModName = new TextEncoder("utf-8").encode(modName)
        return encodedModName.length
    },
    modToLoadNameCopy: function (ptr) {
        let modName = Module.modsToLoad[0].modName;
        let encodedModName = new TextEncoder("utf-8").encode(modName)
        HEAPU8.set(encodedModName, ptr)
    },
    modToLoadDataSize: function () {
        let modData = Module.modsToLoad[0].data;
        return modData.byteLength
    },
    modToLoadDataCopy: function (ptr) {
        HEAPU8.set(Module.modsToLoad[0].data, ptr)
        Module.modsToLoad.shift()
    },
});
