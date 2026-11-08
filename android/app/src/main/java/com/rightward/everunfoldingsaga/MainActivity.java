package com.rightward.everunfoldingsaga;

import android.app.AlertDialog;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.os.Looper;
import android.util.Log;
import android.text.Editable;
import android.text.InputType;
import android.text.TextWatcher;
import android.view.KeyEvent;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.widget.EditText;
import android.widget.TextView;
import android.widget.Toast;

import androidx.activity.OnBackPressedCallback;
import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.view.WindowInsetsControllerCompat;

import com.google.androidgamesdk.GameActivity;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

public class MainActivity extends GameActivity {
    private static final String TAG = "EverUnfoldingSaga";
    private static final String GAME_DIRECTORY_NAME = "The Ever-Unfolding Saga";
    private static final int REQUEST_CODE_IMPORT_DOCUMENT = 1001;
    private static final int REQUEST_CODE_EXPORT_DOCUMENT = 1002;
    private static final int ANDROID_TEXT_INPUT_TARGET_NONE = -1;

    private AndroidTextInputEditText androidTextInput;
    private int androidTextInputTarget = ANDROID_TEXT_INPUT_TARGET_NONE;
    private boolean suppressAndroidTextInputCallbacks;
    private String pendingImportKind;
    private String pendingExportPath;

    static {
        System.loadLibrary("ever_unfolding_saga");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        prepareBundledAssets();
        super.onCreate(savedInstanceState);
        initializeAndroidTextInput();
        getOnBackPressedDispatcher().addCallback(this, new OnBackPressedCallback(true) {
            @Override
            public void handleOnBackPressed() {
                if (!dismissAndroidTextInput()) {
                    nativeBackNavigation();
                }
            }
        });
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (resultCode != RESULT_OK || data == null || data.getData() == null) {
            pendingImportKind = null;
            pendingExportPath = null;
            return;
        }

        Uri uri = data.getData();
        if (requestCode == REQUEST_CODE_IMPORT_DOCUMENT) {
            String kind = pendingImportKind;
            pendingImportKind = null;
            if (!"mods".equals(kind) && !"saves".equals(kind) && !"spatial_fixtures".equals(kind)) {
                return;
            }
            String extension = "mods".equals(kind) ? ".zip"
                    : "saves".equals(kind) ? ".save" : ".json";
            File directory = new File(
                    new File(getFilesDir(), GAME_DIRECTORY_NAME), "imports/" + kind);
            if (!directory.isDirectory() && !directory.mkdirs()) {
                showStorageError("Could not prepare the internal import folder.");
                return;
            }

            File temporary = null;
            try {
                temporary = File.createTempFile("import-", ".tmp", directory);
                try (InputStream input = getContentResolver().openInputStream(uri);
                     FileOutputStream output = new FileOutputStream(temporary)) {
                    if (input == null) {
                        throw new IOException("The selected document could not be opened.");
                    }
                    byte[] buffer = new byte[8192];
                    long copied = 0;
                    long maximum = "mods".equals(kind) ? 128L * 1024 * 1024 : 8L * 1024 * 1024;
                    int length;
                    while ((length = input.read(buffer)) != -1) {
                        copied += length;
                        if (copied > maximum) {
                            throw new IOException("The selected file exceeds the import size limit.");
                        }
                        output.write(buffer, 0, length);
                    }
                    output.getFD().sync();
                }
                File completed = new File(directory, "incoming-" + System.currentTimeMillis() + extension);
                if (completed.exists() || !temporary.renameTo(completed)) {
                    throw new IOException("Could not finalize the imported document.");
                }
                Toast.makeText(this, "Document received. The game will validate it.", Toast.LENGTH_SHORT).show();
            } catch (IOException | SecurityException exception) {
                Log.w(TAG, "Could not import selected document", exception);
                showStorageError("Could not import the selected document: " + exception.getMessage());
                if (temporary != null && temporary.exists()) {
                    temporary.delete();
                }
            }
            return;
        }

        if (requestCode == REQUEST_CODE_EXPORT_DOCUMENT) {
            String sourcePath = pendingExportPath;
            pendingExportPath = null;
            if (sourcePath == null) {
                return;
            }
            File source = new File(sourcePath);
            try (InputStream input = new FileInputStream(source);
                 OutputStream output = getContentResolver().openOutputStream(uri, "wt")) {
                if (output == null) {
                    throw new IOException("The selected destination could not be opened.");
                }
                byte[] buffer = new byte[8192];
                int length;
                while ((length = input.read(buffer)) != -1) {
                    output.write(buffer, 0, length);
                }
                output.flush();
                Toast.makeText(this, "Export completed.", Toast.LENGTH_SHORT).show();
            } catch (IOException | SecurityException exception) {
                Log.w(TAG, "Could not export document", exception);
                showStorageError("Could not export the selected document: " + exception.getMessage());
            }
        }
    }

    /**
     * Finish the Android Activity from the native game's Quit action.
     *
     * Bevy's Android integration keeps its activity handle process-global, while
     * android-activity ties that handle to the current Activity instance. A
     * finished Activity must therefore be followed by a fresh process before the
     * launcher creates another game instance.
     */
    private static native void nativeBackNavigation();
    private static native void nativeAndroidTextInputChanged(int target, String text);
    private static native void nativeAndroidTextInputSubmitted(int target);
    private static native void nativeAndroidTextInputDismissed(int target);

    private final class AndroidTextInputEditText extends EditText {
        AndroidTextInputEditText(MainActivity context) {
            super(context);
        }

        @Override
        public boolean onKeyPreIme(int keyCode, KeyEvent event) {
            if (keyCode == KeyEvent.KEYCODE_BACK
                    && event.getAction() == KeyEvent.ACTION_UP
                    && androidTextInputTarget != ANDROID_TEXT_INPUT_TARGET_NONE) {
                dismissAndroidTextInput();
                return true;
            }
            return super.onKeyPreIme(keyCode, event);
        }
    }

    private void initializeAndroidTextInput() {
        View contentView = findViewById(android.R.id.content);
        if (!(contentView instanceof android.view.ViewGroup)) {
            Log.w(TAG, "Could not initialize Android text input: content view is not a ViewGroup");
            return;
        }

        AndroidTextInputEditText input = new AndroidTextInputEditText(this);
        input.setSingleLine(true);
        input.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS);
        input.setImeOptions(
                EditorInfo.IME_ACTION_DONE
                        | EditorInfo.IME_FLAG_NO_EXTRACT_UI
                        | EditorInfo.IME_FLAG_NO_FULLSCREEN);
        input.setTextColor(0x00000000);
        input.setBackgroundColor(0x00000000);
        input.setCursorVisible(false);
        input.setAlpha(0.0f);
        input.setFocusable(true);
        input.setFocusableInTouchMode(true);
        input.setImportantForAutofill(View.IMPORTANT_FOR_AUTOFILL_NO);
        input.addTextChangedListener(new TextWatcher() {
            @Override
            public void beforeTextChanged(CharSequence source, int start, int count, int after) {
            }

            @Override
            public void onTextChanged(CharSequence source, int start, int before, int count) {
            }

            @Override
            public void afterTextChanged(Editable editable) {
                if (!suppressAndroidTextInputCallbacks
                        && androidTextInputTarget != ANDROID_TEXT_INPUT_TARGET_NONE) {
                    nativeAndroidTextInputChanged(androidTextInputTarget, editable.toString());
                }
            }
        });
        input.setOnEditorActionListener(new TextView.OnEditorActionListener() {
            @Override
            public boolean onEditorAction(
                    TextView view,
                    int actionId,
                    KeyEvent event) {
                if (androidTextInputTarget == ANDROID_TEXT_INPUT_TARGET_NONE) {
                    return false;
                }

                boolean enterKey =
                        event != null
                                && event.getKeyCode() == KeyEvent.KEYCODE_ENTER
                                && event.getAction() == KeyEvent.ACTION_UP;
                if (actionId == EditorInfo.IME_ACTION_DONE || enterKey) {
                    nativeAndroidTextInputSubmitted(androidTextInputTarget);
                    return true;
                }
                return false;
            }
        });

        android.view.ViewGroup.LayoutParams params =
                new android.view.ViewGroup.LayoutParams(1, 1);
        ((android.view.ViewGroup) contentView).addView(input, params);
        androidTextInput = input;
    }

    public void setAndroidTextInput(int target, String text) {
        Runnable update = () -> setAndroidTextInputOnUiThread(target, text);
        if (Looper.myLooper() == Looper.getMainLooper()) {
            update.run();
        } else {
            runOnUiThread(update);
        }
    }

    private void setAndroidTextInputOnUiThread(int target, String text) {
        if (androidTextInput == null) {
            return;
        }

        androidTextInputTarget = target;
        suppressAndroidTextInputCallbacks = true;
        androidTextInput.setText(text == null ? "" : text);
        androidTextInput.setSelection(androidTextInput.length());
        suppressAndroidTextInputCallbacks = false;

        if (target == ANDROID_TEXT_INPUT_TARGET_NONE) {
            InputMethodManager manager =
                    (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
            if (manager != null) {
                manager.hideSoftInputFromWindow(androidTextInput.getWindowToken(), 0);
            }
            androidTextInput.clearFocus();
            return;
        }

        androidTextInput.requestFocus();
        showAndroidTextInputKeyboard();
    }

    private void showAndroidTextInputKeyboard() {
        if (androidTextInput == null
                || androidTextInputTarget == ANDROID_TEXT_INPUT_TARGET_NONE) {
            return;
        }

        androidTextInput.post(() -> {
            if (androidTextInput == null
                    || androidTextInputTarget == ANDROID_TEXT_INPUT_TARGET_NONE) {
                return;
            }

            androidTextInput.requestFocus();

            WindowInsetsControllerCompat insetsController =
                    WindowCompat.getInsetsController(getWindow(), androidTextInput);
            if (insetsController != null) {
                insetsController.show(WindowInsetsCompat.Type.ime());
            }

            InputMethodManager manager =
                    (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
            if (manager != null) {
                manager.restartInput(androidTextInput);
                manager.showSoftInput(
                        androidTextInput,
                        InputMethodManager.SHOW_IMPLICIT);
            }
        });
    }

    private boolean dismissAndroidTextInput() {
        if (androidTextInput == null
                || androidTextInputTarget == ANDROID_TEXT_INPUT_TARGET_NONE) {
            return false;
        }

        int target = androidTextInputTarget;
        androidTextInputTarget = ANDROID_TEXT_INPUT_TARGET_NONE;
        suppressAndroidTextInputCallbacks = true;
        androidTextInput.setText("");
        suppressAndroidTextInputCallbacks = false;

        InputMethodManager manager =
                (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
        if (manager != null) {
            manager.hideSoftInputFromWindow(androidTextInput.getWindowToken(), 0);
        }
        androidTextInput.clearFocus();
        nativeAndroidTextInputDismissed(target);
        return true;
    }

    public void requestGameExit() {
        if (!isFinishing()) {
            finishAndRemoveTask();
        }
    }

    @Override
    protected void onDestroy() {
        boolean finishing = isFinishing();
        super.onDestroy();

        if (finishing) {
            // Defer process termination until the GameActivity destroy callback
            // and the current Android lifecycle callback have fully unwound.
            new android.os.Handler(android.os.Looper.getMainLooper())
                    .post(() -> android.os.Process.killProcess(android.os.Process.myPid()));
        }
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);

        if (hasFocus) {
            hideSystemUi();
            showAndroidTextInputKeyboard();
        }
    }

    public String getNativeGameRootPath() {
        return new File(getFilesDir(), GAME_DIRECTORY_NAME).getAbsolutePath();
    }

    private void prepareBundledAssets() {
        File bundled = new File(new File(getFilesDir(), GAME_DIRECTORY_NAME), "bundled");
        File bundledAssets = new File(bundled, "assets");
        File bundledMods = new File(bundled, "mods");
        if (!bundledAssets.isDirectory() && !bundledAssets.mkdirs()) {
            Log.w(TAG, "Could not create bundled asset directory " + bundledAssets);
        }
        if (!bundledMods.isDirectory() && !bundledMods.mkdirs()) {
            Log.w(TAG, "Could not create bundled mod directory " + bundledMods);
        }
        try {
            copyAssetDirectory("assets", bundledAssets);
            copyAssetDirectory("mods", bundledMods);
        } catch (IOException exception) {
            Log.w(TAG, "Could not prepare bundled game content", exception);
        }
    }

    public void requestStorageImport(String kind) {
        if (!"mods".equals(kind) && !"saves".equals(kind)) {
            showStorageError("Unsupported import type.");
            return;
        }
        pendingImportKind = kind;
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        intent.setType("*/*");
        if ("mods".equals(kind)) {
            intent.putExtra(Intent.EXTRA_MIME_TYPES, new String[] {
                    "application/zip", "application/x-zip-compressed", "application/octet-stream"
            });
        } else if ("saves".equals(kind)) {
            intent.putExtra(Intent.EXTRA_MIME_TYPES, new String[] {
                    "application/gzip", "application/json", "application/octet-stream"
            });
        } else {
            intent.putExtra(Intent.EXTRA_MIME_TYPES, new String[] {
                    "application/json", "application/octet-stream"
            });
        }
        startActivityForResult(intent, REQUEST_CODE_IMPORT_DOCUMENT);
    }

    public void requestStorageExport(String sourcePath, String fileName, String mimeType) {
        if (sourcePath == null || fileName == null || mimeType == null) {
            showStorageError("Invalid export request.");
            return;
        }
        pendingExportPath = sourcePath;
        Intent intent = new Intent(Intent.ACTION_CREATE_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        intent.setType(mimeType);
        intent.putExtra(Intent.EXTRA_TITLE, fileName);
        startActivityForResult(intent, REQUEST_CODE_EXPORT_DOCUMENT);
    }

    private void showStorageError(String message) {
        if (!isFinishing()) {
            new AlertDialog.Builder(this)
                    .setTitle("Storage operation failed")
                    .setMessage(message)
                    .setPositiveButton("Continue", null)
                    .show();
        }
    }

    private void copyAssetDirectory(String assetPath, File destination) throws IOException {
        String[] children = getAssets().list(assetPath);
        if (children == null) {
            return;
        }
        if (children.length == 0) {
            copyAssetFile(assetPath, destination);
            return;
        }
        if (!destination.exists() && !destination.mkdirs()) {
            throw new IOException("Could not create " + destination);
        }
        for (String child : children) {
            File target = new File(destination, child);
            String childAssetPath = assetPath + "/" + child;
            String[] nested = getAssets().list(childAssetPath);
            if (nested != null && nested.length > 0) {
                copyAssetDirectory(childAssetPath, target);
            } else {
                copyAssetFile(childAssetPath, target);
            }
        }
    }

    private void copyAssetFile(String assetPath, File destination) throws IOException {
        File parent = destination.getParentFile();
        if (parent != null && !parent.isDirectory() && !parent.mkdirs()) {
            throw new IOException("Could not create " + parent);
        }
        try (InputStream input = getAssets().open(assetPath);
             FileOutputStream output = new FileOutputStream(destination)) {
            byte[] buffer = new byte[8192];
            int length;
            while ((length = input.read(buffer)) != -1) {
                output.write(buffer, 0, length);
            }
            output.getFD().sync();
        }
    }

    private void hideSystemUi() {
        View decorView = getWindow().getDecorView();
        decorView.setSystemUiVisibility(
                View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                        | View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                        | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                        | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                        | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                        | View.SYSTEM_UI_FLAG_FULLSCREEN
        );
    }
}
