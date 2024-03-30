package org.rpc

import java.net.Socket

fun main(args: Array<String>) {
    val serverAddr = args[0]
    val serverPort = args[1].toInt()
    val command = args.sliceArray(2..<args.size).joinToString(separator = " ") + "\n"

    try {
        val socket = Socket(serverAddr, serverPort)

        Runtime.getRuntime().addShutdownHook(object : Thread(){
            override fun run() {
                socket.shutdownInput()
                socket.shutdownOutput()
                socket.close()
                println("Done")
            }
        })

        socket.outputStream.write(command.toByteArray())
        while (!socket.isClosed) {
            println(socket.inputStream.bufferedReader().readLine())
        }

    } catch (exc: Exception) {
        println(exc.message)
    } finally {

    }
}