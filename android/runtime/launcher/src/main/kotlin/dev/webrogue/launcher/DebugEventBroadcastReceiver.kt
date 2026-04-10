package dev.webrogue.launcher

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

class DebugEventBroadcastReceiver: BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == "dev.webrogue.launcher.DEBUG_EVENT") {
            val data = intent.getStringExtra("data")!!
            this.onData(data)
        }
    }
    companion object {
        init {
            System.loadLibrary("launcher")
        }
    }
    private external fun onData(data: String)
}