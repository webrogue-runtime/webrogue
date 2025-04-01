package io.github.webrogue_runtime.runner;

import android.content.Context;
import android.content.res.AssetFileDescriptor;
import android.os.ParcelFileDescriptor;

import androidx.annotation.Keep;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;

import io.github.webrogue_runtime.common.WebrogueActivity;

public class WebrogueRunnerActivity extends WebrogueActivity {
    @Override
    public boolean exitOnBack() {
        return false;
    }

    @Override
    public void updateContainerFd() {
        Context context = getContext();
        try {
            AssetFileDescriptor afd = context.getAssets().openFd("aot.wrapp");
            containerOffset = afd.getStartOffset();
            containerSize = afd.getLength();
            ParcelFileDescriptor pfd = afd.getParcelFileDescriptor();
            containerFd = pfd.getFd();
            pfd.detachFd();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }
}
