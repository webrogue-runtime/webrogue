addToLibrary({
    presentModSelector: function () {
        Asyncify.handleSleep(wakeUp => {

            var canvas = document.getElementById('canvas');

            Module.runGame = function () {

                for (const child of document.body.children) {
                    child.style.display = "none"
                }
                canvas.style.display = ""
                canvas.focus();
                wakeUp();
                Module.runGame = function () { }
            }

            for (const child of document.body.children) {
                child.style.display = ""
            }
            canvas.style.display = "none";
        })
    },
    prepareMods: function () {
        Asyncify.handleSleep(wakeUp => {
            var transaction = homepageIndexedDB.transaction("mods", 'readonly');
            var allRecords = transaction.objectStore("mods").getAll();
            allRecords.onsuccess = function () {
                Module.modsToLoad = allRecords.result;
                wakeUp();
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
