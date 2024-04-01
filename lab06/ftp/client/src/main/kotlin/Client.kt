package org.ftp

import java.io.File
import java.io.IOException
import java.net.Socket
import java.nio.file.Files
import java.nio.file.Path
import java.util.*
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.io.path.appendBytes
import kotlin.math.min
import kotlin.system.exitProcess


class Client(
    path: String,
    username: String,
    password: String,
    serverAddr: String,
    serverPort: Int
) {

    private companion object {
        const val CMD_CDUP = "CDUP"
        const val CMD_CWD = "CWD"
        const val CMD_DELETE = "DELETE"
        const val CMD_LIST = "LIST"
        const val CMD_MODTIME = "MODTIME"
        const val CMD_MKDIR = "MKDIR"
        const val CMD_PWD = "PWD"
        const val CMD_QUIT = "QUIT"
        const val CMD_DOWNLOAD = "DOWNLOAD"
        const val CMD_STORE = "STORE"
        const val CMD_SIZE = "SIZE"
        const val CMD_RMDIR = "RMDIR"

        const val CRLF = "\r\n"
    }

    private val cwd = Path.of(path)

    private val cmdSocket = Socket(serverAddr, serverPort)
    private val cmdInput = cmdSocket.inputStream.bufferedReader()
    private val cmdOutput = cmdSocket.outputStream


    private val downloader = Executors.newSingleThreadExecutor()

    init {
        val dir = File(cwd.toUri())
        if (!dir.exists() || !dir.isDirectory) {
            println("Invalid working directory: $cwd")
            exitProcess(1)
        }

        waitResponse("220")
        sendCommand("331", "USER $username")
        sendCommand("230", "PASS $password")
    }

    fun start() {
        println(
            """
            Connected to $cmdSocket
            Supported commands:
            $CMD_CDUP --- go to parent directory
            $CMD_CWD <dir> --- change current directory to <dir>
            $CMD_DELETE <filename> --- delete <filename> in current directory
            $CMD_LIST --- list files and directories in current directory
            $CMD_MODTIME <filename> --- get <filename> last modification time
            $CMD_MKDIR <dir> --- create <dir> in current directory
            $CMD_PWD --- get path of current directory 
            $CMD_DOWNLOAD <filename> --- download <filename> from server
            $CMD_SIZE <filename> --- get size of <filename>
            $CMD_STORE <local_filename> --- transfer <local_filename> to current directory on server
            $CMD_RMDIR <dir> --- delete <dir> on server
            $CMD_QUIT --- quit ftp client           
            -----------
        """.trimIndent()
        )

        while (true) {
            val input = readlnOrNull()?.split(" ") ?: continue
            val output = StringBuilder()
            try {
                when (input[0].uppercase(Locale.getDefault())) {
                    CMD_CDUP -> output.append(sendCommand("250", "CDUP"))
                    CMD_CWD -> output.append(sendCommand("250", "CWD ${input[1]}"))
                    CMD_DELETE -> output.append(sendCommand("250", "DELE ${input[1]}"))
                    CMD_MODTIME -> output.append(sendCommand("213", "MDTM ${input[1]}"))
                    CMD_MKDIR -> output.append(sendCommand("257", "MKD ${input[1]}"))
                    CMD_RMDIR -> output.append(sendCommand("250", "RMD ${input[1]}"))
                    CMD_PWD -> output.append(sendCommand("257", "PWD"))
                    CMD_SIZE -> output.append(sendCommand("213", "SIZE ${input[1]}"))

                    CMD_LIST -> {
                        val socket = getPassiveModeSock()
                        output.append(sendCommand("150", "LIST"))
                            .append(socket.inputStream.bufferedReader().readText())
                            .append(waitResponse("226"))
                        socket.close()
                    }

                    CMD_DOWNLOAD -> {
                        val isWorking = AtomicBoolean(true)
                        val socket = getPassiveModeSock()
                        output.append(sendCommand("150", "RETR ${input[1]}"))
                        downloader.submit {
                            val file = Files.createFile(cwd.resolve(input[1]))
                            while (isWorking.get()) {
                                file.appendBytes(socket.inputStream.readAllBytes())
                            }
                        }
                        output.append(waitResponse("226"))
                        isWorking.set(false)
                    }

                    CMD_STORE -> {
                        getPassiveModeSock().use {
                            output.append(sendCommand("150", "STOR ${input[1]}"))
                            val file = File(cwd.resolve(input[1]).toUri())
                            var currLen = 0
                            val fileInputStream = file.inputStream()
                            while (currLen < file.length()) {
                                val currPacket = min(1024 * 1024, file.length() - currLen).toInt()
                                it.outputStream.write(fileInputStream.readNBytes(currPacket))
                                currLen += currPacket
                            }
                        }
                        output.append(waitResponse("226"))
                    }

                    CMD_QUIT -> {
                        cmdSocket.close()
                        output.append("bye")
                        break
                    }

                    else -> throw IOException("Unexpected command: ${input[0]}")
                }
            } catch (exc: Exception) {
                output.appendLine(exc)
            }

            println(output.toString())
        }
    }


    private fun sendCommand(code: String, command: String): String {
        cmdOutput.write((command + CRLF).toByteArray())
        cmdOutput.flush()
        return waitResponse(code)
    }

    private fun waitResponse(code: String): String {
        val result = StringBuilder()
        var response = cmdInput.readLine()
        result.appendLine(response)
        while (response.startsWith("$code-")) {
            response = cmdInput.readLine()
            result.appendLine(response)
        }

        if (!response.startsWith(code)) {
            throw IOException("Unexpected response code: $response\n\tWant: $code")
        }

        return result.toString()
    }

    private fun getPassiveModeSock(): Socket {
        sendCommand("200", "TYPE I")
        sendCommand("200", "MODE S")
        val credits = sendCommand("227", "PASV")
        val dataAddrBytes = credits.dropWhile { it != '(' }.drop(1).takeWhile { it != ')' }.split(",")
        val dataSocket = Socket(
            dataAddrBytes.subList(0, 4).joinToString("."),
            (dataAddrBytes[4].toInt() shl 8) + dataAddrBytes[5].toInt()
        )
        return dataSocket
    }

}
