import org.gradle.api.DefaultTask
import org.gradle.process.ExecOperations

plugins {
    alias(libs.plugins.android.library)
    `maven-publish`
}

abstract class CompileRustTask : DefaultTask() {
    @get:OutputDirectory
    abstract val jniLibsDir: DirectoryProperty

    @get:Inject
    abstract val execOperations: ExecOperations

    @get:Input
    abstract var debuggable: Boolean

    @TaskAction
    fun generate() {
        val libsOut = jniLibsDir.get().asFile
        libsOut.mkdirs()

        execOperations.exec {
            workingDir("../")
            if (debuggable) commandLine(
                "cargo", "ndk",
                "-t", "arm64-v8a",
                "-t", "armeabi-v7a",
                //"-t", "x86",
                //"-t", "x86_64",
                "-o", libsOut.absolutePath,
                "build",
            ) else commandLine(
                "cargo", "ndk",
                "-t", "arm64-v8a",
                "-t", "armeabi-v7a",
                "-t", "x86",
                "-t", "x86_64",
                "-o", libsOut.absolutePath,
                "build",
                "--release"
            )
        }
    }
}

abstract class GenerateUniFfiBindingsTask : DefaultTask() {
    @get:InputFile
    abstract val udl: RegularFileProperty

    @get:OutputDirectory
    abstract val outputDir: DirectoryProperty

    @get:Inject
    abstract val execOperations: ExecOperations

    @TaskAction
    fun generate() {

        execOperations.exec {
            workingDir("../")
            commandLine(
                "cargo",
                "run",
                "--bin",
                "uniffi-bindgen",
                "generate",
                udl.get().asFile.absolutePath,
                "--config",
                "uniffi.toml",
                "--language",
                "kotlin",
                "--out-dir",
                outputDir.get().asFile.absolutePath
            )
        }
    }
}

android {
    namespace = "org.uic.dosipas"
    compileSdk {
        version = release(36) {
            minorApiLevel = 1
        }
    }
    defaultConfig {
        minSdk = 24
    }
    buildTypes {
        release {
            isMinifyEnabled = false
            isShrinkResources = false
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    publishing {
        singleVariant("release") {
            withSourcesJar()
        }
    }
}

androidComponents.onVariants { variant ->
    val compileRust = tasks.register(
        "${variant.name}CompileRust",
        CompileRustTask::class.java
    ) {
        description = "Compile Rust"
        jniLibsDir.set(layout.buildDirectory.dir("generated/jniLibs/${variant.name}"))
        debuggable = variant.debuggable
    }
    val generateBindings = tasks.register(
        "${variant.name}GenerateUniFFIBindings",
        GenerateUniFfiBindingsTask::class.java
    ) {
        description = "UniFFI Bindings"
        udl.set(layout.projectDirectory.file("../src/uic-dosipas.udl"))
        outputDir.set(layout.buildDirectory.dir("generated/source/uniffi/${variant.name}/kotlin"))
    }

    variant.sources.kotlin?.addGeneratedSourceDirectory(generateBindings) { task ->
        task.outputDir
    }

    variant.sources.jniLibs?.addGeneratedSourceDirectory(compileRust) { task ->
        task.jniLibsDir
    }

    variant.sources.kotlin?.addStaticSourceDirectory("./src")
}

dependencies {
    implementation(libs.androidx.appcompat)
    implementation(libs.androidx.core.ktx)
    implementation("net.java.dev.jna:jna:5.19.1@aar")
}

publishing {
    publications {
        create<MavenPublication>("gpr") {
            afterEvaluate {
                from(components["release"])
            }

            groupId = "org.uic"
            artifactId = "dosipas"
            version = "0.0.5"
        }
    }
    repositories {
        maven {
            name = "GitHubPackages"
            url = uri("https://maven.pkg.github.com/TheEnbyperor/uic-dosipas-lib")
            credentials {
                username = project.findProperty("gpr.user") as String? ?: System.getenv("USERNAME")
                password = project.findProperty("gpr.key") as String? ?: System.getenv("TOKEN")
            }
        }
    }
}
