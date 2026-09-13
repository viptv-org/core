plugins {
    kotlin("jvm") version "2.2.0"
    kotlin("plugin.serialization") version "2.2.0"
    `java-library`
}

group = "org.viptv.core.types"
version = "1.0.0"

repositories {
    mavenCentral()
}

dependencies {}

tasks.withType<Jar> {
    manifest {
        attributes["Implementation-Title"] = "org.viptv.core.types"
        attributes["Implementation-Version"] = "1.0.0"
    }
}
