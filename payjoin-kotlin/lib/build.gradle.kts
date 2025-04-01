import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    alias(libs.plugins.kotlin.multiplatform)
    // https://gobley.dev/docs/tutorial#the-cargo-plugin
    alias(libs.plugins.gobley.cargo)
    alias(libs.plugins.gobley.uniffi)
    alias(libs.plugins.kotlin.atomicfu) // Required by Gobley plugin.
    alias(libs.plugins.android.library)
}

android {
    namespace = "org.payjoindevkit"
    compileSdk = 34

    defaultConfig {
        minSdk = 24
    }
}

kotlin {
    androidTarget()

    tasks.withType<KotlinCompile>().configureEach {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
        }
    }
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
}

cargo {
    // The Cargo package with UniFFI bindings is located in parent directory.
    packageDirectory = layout.projectDirectory.dir("../..")

    features.add("uniffi")
}

uniffi {
    generateFromLibrary()
}