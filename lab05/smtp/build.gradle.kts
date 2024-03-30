plugins {
    kotlin("jvm") version "1.9.23"
    application
}

configure<JavaApplication> {
    mainClass.set("org.smtp.MainKt")
}


group = "org.smtp"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    testImplementation("org.jetbrains.kotlin:kotlin-test")
    implementation("org.jetbrains.kotlinx:kotlinx-cli:0.3.6")
    implementation("org.eclipse.angus:angus-mail:2.0.3")
}

tasks.test {
    useJUnitPlatform()
}
kotlin {
    jvmToolchain(21)
}

application {
    mainClass.set("org.smtp.MainKt")
    executableDir = rootDir.path
}