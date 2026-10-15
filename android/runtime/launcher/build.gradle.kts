import com.android.build.api.dsl.ApplicationExtension
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}


configure<ApplicationExtension> {
    namespace = "dev.webrogue.launcher"
    compileSdk = 36

    buildFeatures {
        compose = true
        resValues = true
    }

    defaultConfig {
        applicationId = "dev.webrogue.launcher"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "src/main/kotlin/dev/webrogue/launcher/proguard-wry.pro",
                "proguard-rules.pro"
            )
            isMinifyEnabled = true
            isShrinkResources = true
            signingConfig = signingConfigs.getByName("debug") // TODO remove
            // signingConfig = signingConfigs.getByName("debug")
        }
        debug {
            isDebuggable = true
//            applicationIdSuffix = ".dev"
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlin.compilerOptions {
        jvmTarget = JvmTarget.JVM_11
    }
}

dependencies {
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.webkit)
    implementation(libs.androidx.appcompat)
    implementation(libs.androidx.lifecycle.process)
    implementation(libs.material)
    implementation(libs.androidx.games.activity)
}
