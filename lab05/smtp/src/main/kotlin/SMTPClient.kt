package org.smtp

interface SMTPClient {

    fun sendMail(from: String, to: String, msgSubject: String, msgPath: String)

}