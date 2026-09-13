package tv.viptv.core

import java.io.ByteArrayOutputStream
import java.net.URI
import java.security.SecureRandom
import java.security.cert.X509Certificate
import java.util.concurrent.Executor
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicReference
import javax.net.ssl.SSLContext
import javax.net.ssl.TrustManager
import javax.net.ssl.X509TrustManager
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject

/** Executes only requests emitted by one native SmartCastBridge. */
class SmartCastTransport(
    selectedTvOrigin: String,
    private val executor: Executor,
) {
    data class Response(val status: Int, val body: String)

    class Call internal constructor() {
        internal val cancelled = AtomicBoolean(false)
        internal val connection = AtomicReference<okhttp3.Call?>()
        fun cancel() {
            cancelled.set(true)
            connection.get()?.cancel()
        }
    }

    private val origin = configuredOrigin(selectedTvOrigin)
    private val trustManager = object : X509TrustManager {
        override fun getAcceptedIssuers(): Array<X509Certificate> = emptyArray()
        override fun checkClientTrusted(chain: Array<X509Certificate>, authType: String) = Unit
        override fun checkServerTrusted(chain: Array<X509Certificate>, authType: String) = Unit
    }
    private val sslContext = SSLContext.getInstance("TLS").apply {
        init(null, arrayOf<TrustManager>(trustManager), SecureRandom())
    }
    private val baseClient = OkHttpClient.Builder()
        .sslSocketFactory(sslContext.socketFactory, trustManager)
        .hostnameVerifier { host, _ -> host.equals(URI(origin).host, ignoreCase = true) }
        .followRedirects(false)
        .followSslRedirects(false)
        .retryOnConnectionFailure(false)
        .build()

    /** Callback and transport errors contain no URL, headers, body, token, or certificate data. */
    fun execute(
        request: JSONObject,
        started: (Call) -> Unit = {},
        complete: (Result<Response>) -> Unit,
    ): Call {
        val call = Call()
        val snapshot = JSONObject(request.toString())
        started(call)
        executor.execute { complete(runCatching { exchange(snapshot, call) }) }
        return call
    }

    private fun exchange(request: JSONObject, call: Call): Response {
        check(!call.cancelled.get()) { "SmartCast request cancelled" }
        val uri = URI(request.getString("url"))
        require(exactOrigin(uri) == origin && uri.rawUserInfo == null && uri.rawFragment == null) {
            "SmartCast request is outside the selected TV origin"
        }
        val timeoutMillis = request.getLong("timeoutMillis")
        val maxResponseBytes = request.getLong("maxResponseBytes")
        require(timeoutMillis in 250..60_000 && maxResponseBytes in 1..(8L * 1024 * 1024))
        val builder = Request.Builder().url(uri.toString())
        val headers = request.getJSONObject("headers")
        headers.keys().forEach { name -> builder.header(name, headers.getString(name)) }
        val body = request.optJSONObject("body")?.toString()
        val method = request.getString("method").uppercase()
        val requestBody = body?.toRequestBody("application/json".toMediaType())
        builder.method(method, requestBody)
        val client = baseClient.newBuilder()
            .callTimeout(timeoutMillis, TimeUnit.MILLISECONDS)
            .connectTimeout(timeoutMillis, TimeUnit.MILLISECONDS)
            .readTimeout(timeoutMillis, TimeUnit.MILLISECONDS)
            .build()
        val nativeCall = client.newCall(builder.build())
        call.connection.set(nativeCall)
        check(!call.cancelled.get()) { "SmartCast request cancelled" }
        try {
            nativeCall.execute().use { response ->
                if (response.code in 300..399) error("SmartCast redirects are not permitted")
                val output = ByteArrayOutputStream()
                response.body?.byteStream()?.use { stream ->
                    val buffer = ByteArray(8192)
                    while (true) {
                        check(!call.cancelled.get()) { "SmartCast request cancelled" }
                        val count = stream.read(buffer)
                        if (count < 0) break
                        check(output.size().toLong() + count <= maxResponseBytes) {
                            "SmartCast response exceeds its size limit"
                        }
                        output.write(buffer, 0, count)
                    }
                }
                return Response(response.code, output.toString(Charsets.UTF_8.name()))
            }
        } finally {
            call.connection.set(null)
        }
    }

    private fun exactOrigin(uri: URI): String {
        require(uri.scheme.equals("https", ignoreCase = true))
        require(uri.host != null && uri.rawUserInfo == null && uri.rawQuery == null && uri.rawFragment == null)
        val port = if (uri.port == -1) 443 else uri.port
        return "https://${uri.host.lowercase()}:$port"
    }

    private fun configuredOrigin(input: String): String {
        val uri = URI(if (input.contains("://")) input else "https://$input")
        require(uri.rawPath.isNullOrEmpty() || uri.rawPath == "/")
        val authority = input.substringAfter("://", input).substringBefore('/')
        val explicitPort = if (authority.startsWith("[")) {
            authority.indexOf(']').let { it >= 0 && authority.getOrNull(it + 1) == ':' }
        } else {
            authority.substringAfterLast(':', "").all(Char::isDigit) && authority.contains(':')
        }
        val port = if (explicitPort) uri.port else 7345
        require(port in 1..65535)
        require(uri.scheme.equals("https", ignoreCase = true) && uri.host != null)
        return "https://${uri.host.lowercase()}:$port"
    }
}
