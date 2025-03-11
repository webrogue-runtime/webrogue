package io.github.webrogue_runtime.common;

import android.graphics.Color;
import android.os.Bundle;
import android.os.Process;
import android.view.KeyEvent;
import android.view.ViewGroup;
import android.widget.RelativeLayout;
import android.widget.TextView;

import androidx.annotation.Keep;

import org.libsdl.app.SDLActivity;

import java.nio.charset.StandardCharsets;

public class WebrogueActivity extends SDLActivity {
    private TextView textView;
    private String consoleText = "";
    private String wrappPath = null;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        wrappPath = getIntent().getStringExtra("wrapp_path");
        super.onCreate(savedInstanceState);
        setWindowStyle(true);

        RelativeLayout.LayoutParams layoutParams = new RelativeLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT
        );

        textView = new TextView(this);
        textView.setTextColor(Color.parseColor("#ff000000"));
        layoutParams.addRule(RelativeLayout.ALIGN_TOP);
//        mLayout.addView(textView, layoutParams);
    }

    @Keep
    public String getContainerPath() {
        return wrappPath;
    }

    @Keep
    public void printBytes(byte[] bytes) {
        runOnUiThread(() -> {
            consoleText += new String(bytes, StandardCharsets.UTF_8);
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
