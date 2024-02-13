package io.github.webrogue_runtime;

import android.os.Bundle;
import android.support.annotation.Keep;

import org.libsdl.app.SDLActivity;
import java.io.ByteArrayOutputStream;
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
