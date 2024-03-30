package org.rpc

fun main() {
    val command = "ping yandex.ru"
    val proc = ProcessBuilder(command.split(" ")).start()

    while (proc.isAlive) {
        println(proc.inputStream.bufferedReader().readLine())
    }
}