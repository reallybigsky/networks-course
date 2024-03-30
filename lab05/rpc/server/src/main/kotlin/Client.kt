package org.rpc

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import java.io.BufferedReader
import java.io.InputStreamReader
import java.net.Socket

class Client(private val socket: Socket) {

    fun start(): Unit = runBlocking {
        withContext(Dispatchers.IO) {
            println("Started client $socket")
            socket.soTimeout = 1000
            val command = BufferedReader(InputStreamReader(socket.inputStream)).readLine()
            val output = socket.outputStream
            val proc = ProcessBuilder(command.split(" ")).start()
            while (proc.isAlive) {
                try {
                    val tmp = proc.inputStream.bufferedReader().readLine() + "\n"
                    output.write(tmp.toByteArray())
                } catch (exc: Exception) {
                    break
                }
            }
            println("Closed client $socket")
            socket.close()
            proc.destroyForcibly()
        }
    }

}