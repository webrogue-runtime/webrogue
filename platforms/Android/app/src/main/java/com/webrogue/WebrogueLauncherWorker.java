package com.webrogue;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.Arrays;

public class WebrogueLauncherWorker {
    public class Mod {
        public String name;
        public boolean isActive;
        public File file;
    }

    private Mod[] mods;
    private File storageDir;
    private File modsDirectory;
    private File inactiveModsDirectory;

    public WebrogueLauncherWorker(File storageDir) {
        this.storageDir = storageDir;
        modsDirectory = new File(storageDir, "mods");
        if(!modsDirectory.exists()) modsDirectory.mkdir();
        inactiveModsDirectory = new File(storageDir, "inactive_mods");
        if(!inactiveModsDirectory.exists()) inactiveModsDirectory.mkdir();
        refresh();
    }

    public void addModFile(InputStream inputStream, boolean isActive) {
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
        byte[] fileData = buffer.toByteArray();
        int terminatorIndex = -1;
        for(int i = 0; i<fileData.length; i++) {
            if(fileData[i] == '\0') {
                terminatorIndex = i;
                break;
            }
        }
        if(terminatorIndex == -1 || terminatorIndex>128) return;
        String mod_name = new String(Arrays.copyOfRange(fileData, 0, terminatorIndex));
        String filename = mod_name + ".wrmod";
        File targetFile = new File(isActive ? modsDirectory : inactiveModsDirectory, filename);
        File probableDuplicate = new File(isActive ? inactiveModsDirectory : modsDirectory, filename);
        if(probableDuplicate.exists())
            probableDuplicate.delete();
        try {
            FileOutputStream outputStream = new FileOutputStream(targetFile);
            outputStream.write(fileData, 0, fileData.length);
        } catch (FileNotFoundException e) {

        } catch (IOException e) {

        }
        refresh();
    }

    public void refresh() {
        ArrayList<Mod> raw_mods = new ArrayList<>();
        File[] activeModFiles = modsDirectory.listFiles();
        File[] inactiveModFiles = inactiveModsDirectory.listFiles();
        for (File modFile: activeModFiles) addModFileToList(modFile, raw_mods, true);
        for (File modFile: inactiveModFiles) addModFileToList(modFile, raw_mods, false);
        mods = raw_mods.toArray(new Mod[0]);
        Arrays.sort(mods, (mod, t1) -> mod.name.compareTo(t1.name));
    }

    private void addModFileToList(File modFile, ArrayList<Mod> raw_mods, boolean isActive) {
        Mod mod = new Mod();
        mod.file = modFile;
        mod.isActive = isActive;
        mod.name = modFile.getName();
        raw_mods.add(mod);
    }

    public int getModCount() {
        return mods.length;
    }

    public Mod getMod(int index) {
        return mods[index];
    }

    public void setModActive(Mod mod, boolean isActive) {
        File newFile = new File(isActive ? modsDirectory : inactiveModsDirectory, mod.file.getName());
        mod.file.renameTo(newFile);
        mod.file = newFile;
        mod.isActive = isActive;
    }

    public void delete(Mod mod) {
        mod.file.delete();
    }
}
