package org.rpc

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import java.net.ServerSocket


fun main(args: Array<String>): Unit = runBlocking {
    val port: Int = args[0].toInt()
    withContext(Dispatchers.IO) {
        val serverSocket = ServerSocket(port)
        while (true) {
            val clientSocket = serverSocket.accept()
            val client = Client(clientSocket)
            launch {
                client.start()
            }
        }
    }
}