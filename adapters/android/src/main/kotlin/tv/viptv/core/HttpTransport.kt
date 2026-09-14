package tv.viptv.core

import java.io.ByteArrayOutputStream
import java.io.InterruptedIOException
import java.net.URI
import java.util.concurrent.Executor
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicReference
import java.util.concurrent.TimeUnit
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONArray
import org.json.JSONObject

/** Executes the official Crux HTTP JSON protocol. No account or playback policy lives here. */
class HttpTransport(
    allowedOrigins: Set<String>,
    private val executor: Executor,
    private val timeoutMs: Int = 30_000,
    private val maxResponseBytes: Int = 8 * 1024 * 1024,
) {
    private val origins = allowedOrigins.map { value ->
        val uri = URI(value)
        require(uri.rawPath.isNullOrEmpty() || uri.rawPath == "/")
        require(uri.rawQuery == null && uri.rawFragment == null && uri.rawUserInfo == null)
        origin(uri)
    }.toSet()

    init { require(timeoutMs > 0 && maxResponseBytes > 0) }

    private val client = OkHttpClient.Builder()
        .followRedirects(false).followSslRedirects(false)
        .callTimeout(timeoutMs.toLong(), TimeUnit.MILLISECONDS)
        .connectTimeout(timeoutMs.toLong(), TimeUnit.MILLISECONDS)
        .readTimeout(timeoutMs.toLong(), TimeUnit.MILLISECONDS)
        .retryOnConnectionFailure(false)
        .build()

    /** Cancellable handle shared with SmartCastTransport; it must hold no transport state. */
    class Call internal constructor() {
        internal val cancelled = AtomicBoolean(false)
        internal val connection = AtomicReference<okhttp3.Call?>()
        fun cancel() {
            cancelled.set(true)
            connection.get()?.cancel()
        }
    }

    /** Callback runs on the supplied executor; shell marshals resolve/render to its owner thread. */
    fun execute(request: JSONObject, complete: (JSONObject) -> Unit): Call {
        val call = Call()
        // Snapshot before scheduling; mutable shell JSON must not race the transport.
        val snapshot = JSONObject(request.toString())
        executor.execute {
            val result = exchange(snapshot, call)
            complete(result)
        }
        return call
    }

    private fun exchange(request: JSONObject, call: Call): JSONObject {
        try {
            if (call.cancelled.get()) return error("Io", "Request cancelled")
            val uri = try { URI(request.getString("url")) } catch (_: Exception) {
                return error("Url", "Invalid request URL")
            }
            val approved = try { origin(uri) in origins } catch (_: Exception) { false }
            if (!approved || uri.rawUserInfo != null || uri.rawFragment != null)
                return error("Url", "Request URL is outside the allowed origins")
            val builder = Request.Builder().url(uri.toString())
            val headers = request.getJSONArray("headers")
            for (i in 0 until headers.length()) {
                val header = headers.getJSONObject(i)
                builder.addHeader(header.getString("name"), header.getString("value"))
            }
            val body = request.getJSONArray("body")
            val bytes = ByteArray(body.length()) { i ->
                val byte = body.getInt(i)
                require(byte in 0..255)
                byte.toByte()
            }
            val method = request.getString("method").uppercase()
            val needsBody = method in setOf("POST", "PUT", "PATCH", "PROPPATCH", "REPORT")
            builder.method(method, if (bytes.isNotEmpty() || needsBody) bytes.toRequestBody() else null)
            val nativeCall = client.newCall(builder.build())
            call.connection.set(nativeCall)
            if (call.cancelled.get()) { nativeCall.cancel(); return error("Io", "Request cancelled") }
            nativeCall.execute().use { response ->
                val status = response.code
                if (status in 300..399 && response.header("Location") != null)
                    return error("Url", "Redirects are not permitted")
                val responseHeaders = JSONArray()
                for (i in 0 until response.headers.size) {
                    responseHeaders.put(JSONObject().put("name", response.headers.name(i))
                        .put("value", response.headers.value(i)))
                }
                val output = ByteArrayOutputStream()
                response.body?.byteStream()?.use { stream ->
                    val buffer = ByteArray(8192)
                    while (true) {
                        if (call.cancelled.get()) return error("Io", "Request cancelled")
                        val count = stream.read(buffer)
                        if (count < 0) break
                        if (output.size().toLong() + count > maxResponseBytes)
                            return error("Io", "HTTP response exceeds the size limit")
                        output.write(buffer, 0, count)
                    }
                }
                if (call.cancelled.get()) return error("Io", "Request cancelled")
                val responseBody = JSONArray()
                output.toByteArray().forEach { responseBody.put(it.toInt() and 255) }
                return JSONObject().put("Ok", JSONObject()
                    .put("status", status).put("headers", responseHeaders).put("body", responseBody))
            }
        } catch (_: InterruptedIOException) {
            return if (call.cancelled.get()) error("Io", "Request cancelled")
                else JSONObject().put("Err", "Timeout")
        } catch (_: Exception) {
            return error("Io", if (call.cancelled.get()) "Request cancelled" else "HTTP transport failed")
        } finally {
            call.connection.set(null)
        }
    }

    private fun error(kind: String, message: String) =
        JSONObject().put("Err", JSONObject().put(kind, message))

    private fun origin(uri: URI): String {
        val scheme = uri.scheme?.lowercase()
        require(scheme == "https" || scheme == "http")
        val host = requireNotNull(uri.host).lowercase()
        val port = if (uri.port == -1) { if (scheme == "https") 443 else 80 } else uri.port
        return "$scheme://$host:$port"
    }
}
