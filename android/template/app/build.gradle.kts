plugins {
    alias(libs.plugins.android.application)
}

val isSigning = project.hasProperty("webrogueKeystore") || project.hasProperty("webrogueStorePassword") || project.hasProperty("webrogueKeyPassword") || project.hasProperty("webrogueKeyAlias")

android {
    namespace = "dev.webrogue.runner"
    compileSdk = 36

    signingConfigs {
        if(isSigning) {
            create("singing") {
                storeFile = project.file(project.property("webrogueKeystore") as String)
                storePassword = project.property("webrogueStorePassword") as String
                keyAlias = project.property("webrogueKeyAlias") as String
                keyPassword = project.property("webrogueKeyPassword") as String
            }
        }
    }

    defaultConfig {
        resValue(
            "string",
            "app_name",
            project.property("webrogueApplicationName") as String
        )
        applicationId = project.property("webrogueApplicationId") as String
        minSdk = 24
        targetSdk = 36
        versionCode = (project.property("webrogueVersionCode") as String).toInt()
        versionName = project.property("webrogueVersionName") as String
        if(isSigning) {
            signingConfig = signingConfigs.getByName("singing")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
        }
        debug {
            isDebuggable = true
            applicationIdSuffix = ".dev"
        }
    }


    androidResources {
        noCompress.add("swrapp")
    }
}
