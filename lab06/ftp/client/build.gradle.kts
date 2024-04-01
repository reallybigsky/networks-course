plugins {
    kotlin("jvm")
    application
}

group = "org.ftp"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    testImplementation("org.jetbrains.kotlin:kotlin-test")
    implementation("org.jetbrains.kotlinx:kotlinx-cli:0.3.6")
}

tasks.test {
    useJUnitPlatform()
}
kotlin {
    jvmToolchain(21)
}

application {
    mainClass.set("org.ftp.MainKt")
}

tasks.named<JavaExec>("run") {
    standardInput = System.`in`
}