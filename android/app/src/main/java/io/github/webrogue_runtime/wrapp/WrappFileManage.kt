package io.github.webrogue_runtime.wrapp

import java.io.File
import java.io.InputStream
import java.security.MessageDigest

class WrappFileManage(private val dataDir: File) {
    fun listMods(): List<WrappRef> {
        val filenames = dataDir.list { _, s -> s.endsWith(".wrapp") }?.asList().orEmpty()
        val refs = filenames.mapNotNull { filename -> resolveFileRef(File(dataDir, filename)) }
        return refs
    }

    fun storeWrappFromStream(inputStream: InputStream): WrappRef? {
        val file = File(dataDir, "tmp_wrapp")
        file.delete()
        file.createNewFile()
        inputStream.copyTo(file.outputStream())
        inputStream.close()
        var ref = resolveFileRef(file)
        if(ref == null) {
            file.delete()
            return null
        }
        val renamedFile = File(dataDir, "${ref.sha}.wrapp")
        file.renameTo(renamedFile);
        ref = resolveFileRef(renamedFile)
        if(ref == null) {
            renamedFile.delete()
            return null
        }
        return ref
    }

    @OptIn(ExperimentalStdlibApi::class)
    private fun resolveFileRef(file: File): WrappRef? {
        val fileStream = file.inputStream()
        var byteArray = ByteArray(6)
        fileStream.read(byteArray)
        val magic = "WRAPP".encodeToByteArray().plus(0)
        if(!byteArray.contentEquals(magic)) {
            file.delete()
            return null
        }
        val sha = MessageDigest.getInstance("SHA-256")
        sha.update(magic)
        byteArray = ByteArray(1024)
        while(true) {
            val nBytes = fileStream.read(byteArray)
            if(nBytes <= 0) break
            sha.update(byteArray.sliceArray(0..<nBytes))
        }
        val hash = sha.digest().toHexString()
        return WrappRef(hash, file.path)
    }
}
