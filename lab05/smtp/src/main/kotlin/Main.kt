package org.smtp

import kotlinx.cli.*
import java.io.IOException

fun main(args: Array<String>) {
    val parser = ArgParser("smtp_client")

    val client by parser.option(ArgType.String, description = "type of smtp client. Either \"simple\" or \"socket\" value").default("simple")
    val from by parser.option(ArgType.String, description = "sender email address").required()
    val username by parser.option(ArgType.String, description = "sender username").required()
    val password by parser.option(ArgType.String, description = "sender email password").required()
    val to by parser.option(ArgType.String, description = "recipient email address").required()
    val messagePath by parser.option(ArgType.String, fullName = "message-filepath", description = "path file with message").required()
    val messageSubject by parser.option(ArgType.String, fullName = "message-subject", description = "message subject").default("Message subject")
    val serverAddr by parser.option(ArgType.String, fullName = "server-addr", description = "smtp server address").default("smtp.yandex.ru")
    val serverPort by parser.option(ArgType.Int, fullName = "server-port", description = "smtp server port").default(587)

    parser.parse(args)

    try {
        val cl = when (client) {
            "simple" -> SimpleClient(username, password, serverAddr, serverPort)
            "socket" -> SocketClient(username, password, serverAddr, serverPort)
            else -> throw IOException("Unexpected client value: $client")
        }
        cl.sendMail(from, to, messageSubject, messagePath)
    } catch (exc: Exception) {
        println("Error: ${exc.message}")
    }

    println("Email sent!")
}