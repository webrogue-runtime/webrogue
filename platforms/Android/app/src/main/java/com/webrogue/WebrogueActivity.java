package com.webrogue;

import android.Manifest;
import android.content.Context;
import android.content.pm.PackageManager;
import android.os.Build;
import android.os.Bundle;
import android.os.Environment;
import android.support.annotation.Keep;
import android.view.View;

import org.libsdl.app.SDLActivity;
import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileFilter;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.FilenameFilter;
import java.io.IOException;
import java.io.InputStream;

public class WebrogueActivity extends SDLActivity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        sharedWebrogueActivity = this;
        super.onCreate(savedInstanceState);
        // setWindowStyle(true);
    }

    private static WebrogueActivity sharedWebrogueActivity;

    private String getStoragePath() {
        return getFilesDir().getAbsolutePath();
    }

    @Keep
    public static String staticGetStoragePath() {
        return sharedWebrogueActivity.getStoragePath();
    }
    private byte[] getCoreData(int resource) {
        InputStream inputStream = getResources().openRawResource(resource);
        ByteArrayOutputStream buffer = new ByteArrayOutputStream();
        int nRead;
        byte[] readBuffer = new byte[256];
        try {
            while ((nRead = inputStream.read(readBuffer, 0, readBuffer.length)) != -1) {
                buffer.write(readBuffer, 0, nRead);
            }
            buffer.flush();
        } catch (IOException e) {

        }
        return buffer.toByteArray();
    }
    @Keep
    public static byte[] staticGetCoreData() {
        return sharedWebrogueActivity.getCoreData(R.raw.core);
    }
    @Override
    protected String[] getLibraries() {
        return new String[]{ "webrogue" };
    }
}
