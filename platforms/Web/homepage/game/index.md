---
layout: page
title: Dynamic demo
---

<div style="display: none;"> <!-- invisible container fore reusable elements -->
    <table>
        <tr id="exampleModItem">
            <td>
                <label id="modItemLabel">Example mod item</label>
            </td>
            <td>
                <button type="button" id="downloadButton"><label>Download</label></button>
                <button type="button" id="deleteButton"><label>Delete</label></button>
            </td>
        </tr>
    </table>
</div>

<div id="noJavaScript">
    Warning! You browser does not supports JavaScript or it is turned off. This demo will not work.
    <br/><br/>
</div>
<script type='text/javascript'>
    document.getElementById('noJavaScript').style.display = "none";
</script>

<div style="display:none" id="noWebAssembly">
    Warning! You browser does not supports WebAssembly or it is turned off. This demo will not work.
    <br/><br/>
</div>


<div style="display:none" id="noSharedArrayBuffer">
    Warning! You browser has no SharedArrayBuffer support or it is turned off. This demo will not work.
    <br/><br/>
</div>



This page shows how mods can be added to Webrogue [on the fly](../benefits/portable).
You can upload a mod using this input: 
<input type="file" id="fileSelector" />
It should appear on table below.

<table id="modSelector"></table>

Note that if no mods are provided then [log2048](../examples/log2048) will be used by default.
To remove installed mod, click "Delete" button. When you are ready to launch a game, click this button:
<button type="button" onclick="Module.runGame()">Run</button>.

Detailed explanation how mods are executed on web is [here](../in_depth/web_runtime).


<script type='text/javascript'>
    if(typeof WebAssembly === 'undefined')
        document.getElementById("noWebAssembly").style.display = null
    if(typeof SharedArrayBuffer === 'undefined')
        document.getElementById("noSharedArrayBuffer").style.display = null
    var predefinedMods = [
        { modName: "log2048", url: "./artifacts/mods/log2048.wrmod" },
    ];
    var storedMods = [];
    var modSelectorElement = document.getElementById("modSelector");

    var homepageIndexedDB = undefined;

    function reloadModList() {
        var transaction = homepageIndexedDB.transaction("mods", 'readonly');
        var allRecords = transaction.objectStore("mods").getAll();
        allRecords.onsuccess = function () {
            setStoredMods(allRecords.result);
        };
    }

    const fileSelector = document.getElementById('fileSelector');
    fileSelector.value = null;
    fileSelector.addEventListener('change', (fileSelectorEvent) => {
        var remainFiles = fileSelectorEvent.target.files.length;
        Array.from(fileSelectorEvent.target.files).forEach((file) => {
            var reader = new FileReader();
            reader.readAsArrayBuffer(file);
            reader.onload = function (event) {
                remainFiles--;
                if (remainFiles == 0) {
                    fileSelector.value = null;
                }
                var transaction = homepageIndexedDB.transaction("mods", 'readwrite');
                transaction.oncomplete = function (event) {
                    reloadModList();
                };
                var bytes = new Uint8Array(event.target.result);

                var nameBytes = bytes.slice(0, bytes.findIndex((byte) => byte == 0));
                var modName = (new TextDecoder()).decode(nameBytes);
                var allRecords = transaction.objectStore("mods").put({ modName: modName, data: bytes });
            }
            reader.onerror = function (event) {
                remainFiles--;
                console.error("error while uploading file")
            }
        });
    });


    function setStoredMods(newStoredMods) {
        storedMods = newStoredMods;
        predefinedMods.filter((predefinedMod) => {
            var result = true;
            storedMods.forEach(storedMod => {
                if (storedMod.modName == predefinedMod.modName)
                    result = false;
            });
            return result;
        }).forEach((predefinedMod) => {
            storedMods.unshift(predefinedMod)
        })
        while (modSelectorElement.firstChild) {
            modSelectorElement.removeChild(modSelectorElement.lastChild);
        }
        storedMods.forEach((mod) => {
            var newNode = document.getElementById("exampleModItem").cloneNode(true);
            newNode.querySelector("#modItemLabel").textContent = mod.data ? mod.modName : mod.modName+", not installed";
            var downloadButton = newNode.querySelector("#downloadButton");
            if (mod.data)
                downloadButton.style.display = "none";
            else
                downloadButton.onclick = function () {
                    fetch(mod.url).then(function (response) {
                        if (!response.ok) {
                            return false;
                        }
                        return response.blob();
                    }).then(function (myBlob) {
                        myBlob.arrayBuffer().then((content) => {
                            var transaction = homepageIndexedDB.transaction("mods", 'readwrite');
                            transaction.oncomplete = function (event) {
                                reloadModList();
                            };
                            transaction.objectStore("mods").add({ modName: mod.modName, data: new Uint8Array(content) });
                        });
                    });
                }
            var deleteButton = newNode.querySelector("#deleteButton");
            if (!mod.data)
                deleteButton.style.display = "none";
            else
                deleteButton.onclick = function () {
                    var transaction = homepageIndexedDB.transaction("mods", 'readwrite');
                    transaction.oncomplete = function (event) {
                        reloadModList();
                    };
                    transaction.objectStore("mods").delete(mod.modName);
                }
            modSelectorElement.appendChild(newNode);
        })
    }


    var request = window.indexedDB.open("webrogueHomepage", 1);
    request.onerror = (event) => {
        console.error(`Database error: ${event.target.errorCode}`);
        alert(`Database error: ${event.target.errorCode}`);
    };
    request.onupgradeneeded = (event) => {
        var db = event.target.result;
        var objectStore = db.createObjectStore("mods", { keyPath: "modName" });

        objectStore.createIndex("modName", "modName", { unique: true });
    }
    request.onsuccess = (event) => {
        homepageIndexedDB = event.target.result;
        reloadModList();
    };
</script>

<div style="position: absolute;left: 0;right: 0;top: 0;">
    <div class="emscripten">
        <label id="statusLabel" style="display: none;"></label>
    </div>
</div>

<canvas style="height: 100vh;left: 0; top: 0; width:100%" id="canvas" oncontextmenu="event.preventDefault()"  > 
</canvas>

<script type='text/javascript'>
window.coi = {
    doReload: () => window.location.reload(),
}
</script>

<script src="coi-serviceworker.js"></script>

<script type='text/javascript'>
    var canvas = document.getElementById('canvas')
    canvas.style.display = "none";
    document.body.appendChild(canvas)
    // if (!crossOriginIsolated)
    //     window.location.reload();

    var statusLabelElement = document.getElementById('statusLabel');

    var Module = {
        preRun: [
            function () {
                FS.mkdir('/webrogue');
                FS.mount(IDBFS, { root: '/' }, '/webrogue');
                FS.chdir('/webrogue');
                FS.syncfs(true, function (err) {
                    // handle callback
                });
            }
        ],
        postRun: [],
        print: (function () {
            return function (text) {
                if (arguments.length > 1) text = Array.prototype.slice.call(arguments).join(' ');
                console.log(text);
            };
        })(),
        canvas: (function () {
            var canvas = document.getElementById('canvas');

        // As a default initial behavior, pop up an alert when webgl context is lost. To make your
        // application robust, you may want to override this behavior before shipping!
        // See http://www.khronos.org/registry/webgl/specs/latest/1.0/#5.15.2
            canvas.addEventListener("webglcontextlost", function (e) { alert('WebGL context lost. You will need to reload the page.'); e.preventDefault(); }, false);

            return canvas;
        })(),
        setStatus: function (text) {
            if (text == "Running...") text = "Loading Webrogue...";
            statusLabelElement.textContent = text
        },
        totalDependencies: 0,
    };
    Module.setStatus('Downloading...');
    window.onerror = function (event) {
        // TODO: do not warn on ok events like simulating an infinite loop or exitStatus
        Module.setStatus('Exception thrown, see JavaScript console');
        Module.setStatus = function (text) {
            if (text) console.error('[post-exception status] ' + text);
        };
    };
</script>
<script async type="text/javascript" src="artifacts/webrogue_game.js"></script>
