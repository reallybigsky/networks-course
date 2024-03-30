package org.smtp

import jakarta.mail.*
import jakarta.mail.internet.InternetAddress
import jakarta.mail.internet.MimeBodyPart
import jakarta.mail.internet.MimeMessage
import jakarta.mail.internet.MimeMultipart
import java.io.File
import java.io.IOException
import java.nio.file.Files
import java.util.*


class SimpleClient(username: String, password: String, serverAddr: String, serverPort: Int) : SMTPClient {

    private val props = Properties()
    private val session: Session

    init {
        props["mail.smtp.auth"] = true
        props["mail.smtp.starttls.enable"] = "true"
        props["mail.smtp.host"] = serverAddr
        props["mail.smtp.port"] = serverPort.toString()
        props["mail.smtp.ssl.trust"] = serverAddr
        session = Session.getInstance(props, object : Authenticator() {
            override fun getPasswordAuthentication(): PasswordAuthentication {
                return PasswordAuthentication(username, password)
            }
        })
    }

    override fun sendMail(from: String, to: String, msgSubject: String, msgPath: String) {
        val message = MimeMessage(session)
        message.setFrom(InternetAddress(from))
        message.setRecipients(Message.RecipientType.TO, InternetAddress.parse(to))
        message.subject = msgSubject

        val msgFile = File(msgPath)
        val msgMimeType = Files.probeContentType(msgFile.toPath())
        if (!msgMimeType.startsWith("text")) {
            throw IOException("Cannot send MIME type: $msgMimeType")
        }

        val msg = File(msgPath).readText(Charsets.UTF_8)

        val mimeBodyPart = MimeBodyPart()
        mimeBodyPart.setContent(msg, "$msgMimeType; charset=utf-8")

        val multipart: Multipart = MimeMultipart()
        multipart.addBodyPart(mimeBodyPart)

        message.setContent(multipart)

        Transport.send(message)
    }

}