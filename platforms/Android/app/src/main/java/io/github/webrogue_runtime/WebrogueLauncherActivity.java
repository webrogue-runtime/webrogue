package io.github.webrogue_runtime;

import android.app.Activity;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.provider.OpenableColumns;
import android.support.v7.app.AppCompatActivity;
import android.os.Bundle;
import android.support.v7.widget.LinearLayoutManager;
import android.support.v7.widget.RecyclerView;
import android.view.Menu;
import android.view.MenuInflater;
import android.view.MenuItem;

import java.io.FileNotFoundException;
import java.io.InputStream;

public class WebrogueLauncherActivity extends AppCompatActivity {
    protected WebrogueLauncherWorker worker;
    protected RecyclerView mRecyclerView;
    protected ModListAdapter adapter;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_webrogue_launcher);
        worker = new WebrogueLauncherWorker(getFilesDir());
        adapter = new ModListAdapter(worker);
        mRecyclerView = (RecyclerView) findViewById(R.id.recycler_view);
        registerForContextMenu(mRecyclerView);
        mRecyclerView.setLayoutManager( new LinearLayoutManager(this));
        mRecyclerView.setAdapter(adapter);
        if(worker.getModCount() == 0)
            addLog2048();
        Intent intent = getIntent();
        String action = intent.getAction();
        String type = intent.getType();

        if (Intent.ACTION_VIEW.equals(action) && type != null) {
            Uri uri = intent.getData();
            addModFromUri(uri);
        }


    }

    public boolean onCreateOptionsMenu(Menu menu) {
        MenuInflater inflater = getMenuInflater();
        inflater.inflate(R.menu.launcher, menu);
        return true;
    }

    public void onMenuItemClick(MenuItem item) {
        String id = getResources().getResourceName(item.getItemId()).split("\\/")[1];
        switch (id) {
            case "run":
                Intent myIntent = new Intent(WebrogueLauncherActivity.this, WebrogueActivity.class);
                WebrogueLauncherActivity.this.startActivity(myIntent);
                break;
            case "import_mod":
                Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                intent.addCategory(Intent.CATEGORY_OPENABLE);
                intent.setType("*/*");

                startActivityForResult(intent, PICK_PDF_FILE);
                break;
            case "refresh":
                refresh();
                break;
            case "add_log2048":
                addLog2048();
                break;
            default:
        }
    }
    // Request code for selecting a PDF document.
    private static final int PICK_PDF_FILE = 2;

    @Override
    public void onActivityResult(int requestCode, int resultCode, Intent resultData) {
        if (requestCode == PICK_PDF_FILE && resultCode == Activity.RESULT_OK) {
            // The result data contains a URI for the document or directory that
            // the user selected.
            Uri uri = null;
            if (resultData != null) {
                uri = resultData.getData();
                addModFromUri(uri);
                // Perform operations on the document using its URI.
            }
        } else {
            super.onActivityResult(requestCode, resultCode, resultData);
        }
    }

    protected void addModFromUri(Uri uri) {
        try {
            InputStream stream = getContentResolver().openInputStream(uri);
            worker.addModFile(stream, false);
        } catch (FileNotFoundException e) {};
        refresh();
    }

    protected String getFileName(Uri uri) {
        String result = null;
        if (uri.getScheme().equals("content")) {
            Cursor cursor = getContentResolver().query(uri, null, null, null, null);
            try {
                if (cursor != null && cursor.moveToFirst()) {
                    int columnIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME);
                    if(columnIndex>=0)
                        result = cursor.getString(columnIndex);
                }
            } finally {
                cursor.close();
            }
        }
        if (result == null) {
            result = uri.getPath();
            int cut = result.lastIndexOf('/');
            if (cut != -1) {
                result = result.substring(cut + 1);
            }
        }
        return result;
    }

    public void refresh() {
        worker.refresh();
        adapter.notifyDataSetChanged();
    }

    protected void addLog2048() {
        worker.addModFile(getResources().openRawResource(R.raw.log2048),  true);
        refresh();
    }
}