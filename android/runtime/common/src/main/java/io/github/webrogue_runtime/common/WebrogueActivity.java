package io.github.webrogue_runtime.common;

import android.graphics.Color;
import android.os.Bundle;
import android.os.ParcelFileDescriptor;
import android.os.Process;
import android.util.Log;
import android.view.KeyEvent;
import android.view.ViewGroup;
import android.widget.RelativeLayout;
import android.widget.TextView;

import androidx.annotation.Keep;

import org.libsdl.app.SDLActivity;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.nio.charset.StandardCharsets;

public class WebrogueActivity extends SDLActivity {
    private TextView textView;
    private String consoleText = "";
    private String dataPath = null;

    protected long containerFd;
    protected long containerOffset;
    protected long containerSize;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        dataPath = getFilesDir().getAbsolutePath();
        updateContainerFd();
        setWindowStyle(true);

//        RelativeLayout.LayoutParams layoutParams = new RelativeLayout.LayoutParams(
//                ViewGroup.LayoutParams.MATCH_PARENT,
//                ViewGroup.LayoutParams.WRAP_CONTENT
//        );

//        textView = new TextView(this);
//        textView.setTextColor(Color.parseColor("#ff000000"));
//        layoutParams.addRule(RelativeLayout.ALIGN_TOP);
//        mLayout.addView(textView, layoutParams);
    }

    public void updateContainerFd() {
        String container_path = getIntent().getStringExtra("wrapp_path");
        try {
            ParcelFileDescriptor pfd = ParcelFileDescriptor.open(new File(container_path), ParcelFileDescriptor.MODE_READ_ONLY);
            containerOffset = 0;
            containerSize = pfd.getStatSize();
            containerFd = pfd.getFd();
            pfd.detachFd();
        } catch (FileNotFoundException e) {
            throw new RuntimeException(e);
        }
    }

    @Keep
    public long getContainerFd() {
        return containerFd;
    }

    @Keep
    public long getContainerOffset() {
        return containerOffset;
    }

    @Keep
    public long getContainerSize() {
        return containerSize;
    }

    @Keep
    public String getDataPath() {
        return dataPath;
    }

    @Keep
    public void printBytes(byte[] bytes) {
        String string = new String(bytes, StandardCharsets.UTF_8);
        runOnUiThread(() -> {
            consoleText += string;
            textView.setText(consoleText);
        });
    }

    @Override
    protected String[] getLibraries() {
        return new String[]{"webrogue"};
    }

    @Override
    protected void onDestroy() {
        Process.killProcess(Process.myPid());
        super.onDestroy();
    }

    public boolean exitOnBack() {
        return true;
    }

    @Override
    public boolean dispatchKeyEvent(KeyEvent event) {
        if (event.getKeyCode() == KeyEvent.KEYCODE_BACK && exitOnBack()) {
//            Process.killProcess(Process.myPid());
            this.finishAndRemoveTask();
            return true;
        } else {
            return super.dispatchKeyEvent(event);
        }
    }
}
