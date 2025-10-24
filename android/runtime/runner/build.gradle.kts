plugins {
    alias(libs.plugins.android.application)
}

android {
    namespace = "dev.webrogue.runner"
    compileSdk = 36
    
    signingConfigs {
        create("dummy") {
            // keyAlias keystoreProperties['keyAlias']
            // keyPassword keystoreProperties['keyPassword']
            storeFile = project.file("dummy.jks")
            storePassword = "testtestz"
        }
    }

    defaultConfig {
        resValue("string", "app_name", "Webrogue runner")
        applicationId = "dev.webrogue.runner"
        minSdk = 24
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            signingConfig = signingConfigs.getByName("debug")
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
