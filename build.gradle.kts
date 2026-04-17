import com.diffplug.gradle.spotless.SpotlessExtension

plugins {
    java
    application
    id("com.diffplug.spotless") version "6.25.0"
    id("checkstyle")
    id("com.gradleup.shadow") version "8.3.5"
}

group = "com.aether"
version = "0.1.0-SNAPSHOT"

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(25)
    }
}

repositories {
    mavenCentral()
}

dependencies {
    // JSON (replaces serde_json)
    implementation("com.fasterxml.jackson.core:jackson-databind:2.17.2")

    // REPL line editing (replaces rustyline)
    implementation("org.jline:jline:3.26.3")

    // Lombok: boilerplate reduction
    compileOnly("org.projectlombok:lombok:1.18.42")
    annotationProcessor("org.projectlombok:lombok:1.18.42")
    testCompileOnly("org.projectlombok:lombok:1.18.42")
    testAnnotationProcessor("org.projectlombok:lombok:1.18.42")

    // Testing
    testImplementation("org.junit.jupiter:junit-jupiter:5.11.0")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

application {
    mainClass = "com.aether.Main"
}

tasks.withType<JavaCompile> {
    options.compilerArgs.addAll(listOf("--enable-preview"))
}

tasks.withType<Test> {
    useJUnitPlatform()
    jvmArgs("--enable-preview")
}

tasks.withType<JavaExec> {
    jvmArgs("--enable-preview")
}

// Spotless: Google Java Format
configure<SpotlessExtension> {
    java {
        googleJavaFormat("1.23.0").aosp().reflowLongStrings()
        removeUnusedImports()
        trimTrailingWhitespace()
        endWithNewline()
        importOrder("java", "javax", "org", "com")
        target("src/**/*.java")
    }
}

// Checkstyle
checkstyle {
    toolVersion = "10.18.1"
    configFile = file("checkstyle.xml")
    isIgnoreFailures = false
}

tasks.named("check") {
    dependsOn("spotlessCheck")
}

// Fat JAR: bundle all dependencies for distribution
tasks.named<com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar>("shadowJar") {
    archiveBaseName = "aether"
    archiveClassifier = ""
    archiveVersion = ""
    manifest {
        attributes["Main-Class"] = "com.aether.Main"
        attributes["Multi-Release"] = "true"
    }
    mergeServiceFiles()
}
