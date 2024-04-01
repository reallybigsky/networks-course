package org.ftp

import kotlinx.cli.ArgParser
import kotlinx.cli.ArgType
import kotlinx.cli.default
import kotlinx.cli.required

fun main(args: Array<String>) {
    val parser = ArgParser("FTP client")

    val cwd by parser.option(ArgType.String, description = "local working directory").default("")
    val username by parser.option(ArgType.String, description = "username").required()
    val password by parser.option(ArgType.String, description = "password").default("")
    val serverAddr by parser.option(ArgType.String, fullName = "server-addr", description = "server address").required()
    val serverPort by parser.option(ArgType.Int, fullName = "server-port", description = "server port").required()

    parser.parse(args)

    val client = Client(cwd, username, password, serverAddr, serverPort)
    client.start()
}