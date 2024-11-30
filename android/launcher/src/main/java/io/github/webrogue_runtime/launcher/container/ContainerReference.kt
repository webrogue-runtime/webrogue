package io.github.webrogue_runtime.launcher.container

import java.io.File

class ContainerReference(val sha: String, val path: String) {
    fun delete() {
        File(path).delete()
    }
}