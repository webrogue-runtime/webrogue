package io.github.webrogue_runtime.runner;

import android.content.Context;

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
    @Keep
    public String getContainerPath() {
        Context context = getContext();
        File outputFile = new File(context.getCacheDir(), "aot.wrapp");
        try {
            FileOutputStream outputStream = new FileOutputStream(outputFile);
            InputStream inputStream = context.getAssets().open("aot.wrapp");
            byte[] buffer = new byte[1024];
            int len;
            while ((len = inputStream.read(buffer)) != -1) {
                outputStream.write(buffer, 0, len);
            }
            outputStream.close();
            inputStream.close();
            return outputFile.getPath();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }
}
