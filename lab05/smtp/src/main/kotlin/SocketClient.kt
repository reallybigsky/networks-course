package org.smtp

import jakarta.activation.MimeType
import java.io.*
import java.nio.ByteBuffer
import java.nio.file.Files
import javax.net.ssl.SSLSocket
import javax.net.ssl.SSLSocketFactory
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

class SocketClient(private val username: String, private val password: String, serverAddr: String, serverPort: Int) :
    SMTPClient {

    companion object {
        const val CRLF = "\r\n"
    }

    private val socket = SSLSocketFactory.getDefault().createSocket(serverAddr, serverPort) as SSLSocket
    private val input = BufferedReader(InputStreamReader(socket.inputStream))
    private val output = BufferedOutputStream(socket.outputStream)

    private fun sendRequest(code: String, message: String) {
        output.write(message.toByteArray())
        output.flush()
        while (true) {
            val line = input.readLine()
            when {
                line.startsWith("$code ") -> {
                    System.err.println(line)
                    break

                }

                line.startsWith("$code-") -> {
                    System.err.println(line)
                }

                else -> throw IOException("Invalid response code from server for request: $message \nExpected code $code, got $line")
            }
        }
    }

    @OptIn(ExperimentalEncodingApi::class)
    override fun sendMail(from: String, to: String, msgSubject: String, msgPath: String) {
        if (!input.readLine().startsWith("220")) {
            throw IOException("Invalid cannot handshake with server")
        }

        val msgFile = File(msgPath)
        val msgMimeType = Files.probeContentType(msgFile.toPath())
        var msgBody = msgFile.readBytes()

        sendRequest("250", "HELO org.smtp.socket.client$CRLF")
        sendRequest("334", "AUTH LOGIN$CRLF")
        sendRequest("334", Base64.encode(username.toByteArray()) + CRLF)
        sendRequest("235", Base64.encode(password.toByteArray()) + CRLF)
        sendRequest("250", "MAIL FROM: <$from>$CRLF")
        sendRequest("250", "RCPT TO: <$to>$CRLF")
        sendRequest("354", "DATA$CRLF")

        val builder = StringBuilder()
            .append("From: $from$CRLF")
            .append("To: $to$CRLF")
            .append("Subject: $msgSubject$CRLF")
            .append("Mime-Version: 1.0$CRLF")
            .append("Content-Type: $msgMimeType$CRLF")

        if (!msgMimeType.startsWith("text")) {
            builder.append("Content-Transfer-Encoding: base64$CRLF")
                .append("Content-Disposition: attachment; filename=${msgFile.name}$CRLF")
            msgBody = Base64.encodeToByteArray(msgBody)
        }

        builder.append(CRLF)
        output.write(builder.toString().toByteArray())
        output.write(msgBody)

        sendRequest("250", "$CRLF.$CRLF")
        output.write("QUIT$CRLF".toByteArray())
        output.flush()
    }

}
