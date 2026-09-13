package tv.viptv.core

import java.net.ServerSocket
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test

/** Run in the consuming Android instrumentation target, with loopback cleartext allowed. */
class HttpTransportTest {
    private fun request(url: String) = JSONObject()
        .put("url", url).put("method", "PATCH")
        .put("headers", JSONArray().put(JSONObject().put("name", "X-Fixture").put("value", "present")))
        .put("body", JSONArray().put(0).put(255))

    @Test fun patchBinaryAndErrorStatusSurvive() {
        ServerSocket(0).use { server ->
            server.soTimeout = 5000
            val executor = Executors.newFixedThreadPool(2)
            try {
                val seen = AtomicReference<String>()
                val serverDone = executor.submit {
                    server.accept().use { socket ->
                        socket.soTimeout = 5000
                        val input = socket.getInputStream()
                        val header = StringBuilder()
                        while (!header.endsWith("\r\n\r\n")) {
                            val byte = input.read()
                            check(byte >= 0)
                            header.append(byte.toChar())
                        }
                        seen.set(header.toString())
                        assertEquals(0, input.read())
                        assertEquals(255, input.read())
                        socket.getOutputStream().apply {
                            write("HTTP/1.1 401 Unauthorized\r\nContent-Length: 2\r\nRetry-After: 4\r\nConnection: close\r\n\r\n".toByteArray())
                            write(byteArrayOf(-1, 0))
                            flush()
                        }
                    }
                }
                val origin = "http://127.0.0.1:${server.localPort}"
                val done = CountDownLatch(1)
                val result = AtomicReference<JSONObject>()
                HttpTransport(setOf(origin), executor).execute(request("$origin/api/profile")) {
                    result.set(it); done.countDown()
                }
                assertTrue(done.await(5, TimeUnit.SECONDS))
                serverDone.get(5, TimeUnit.SECONDS)
                assertTrue(seen.get().startsWith("PATCH /api/profile HTTP/1.1"))
                assertTrue(seen.get().contains("X-Fixture: present", ignoreCase = true))
                val response = result.get().getJSONObject("Ok")
                assertEquals(401, response.getInt("status"))
                assertEquals("[255,0]", response.getJSONArray("body").toString())
                assertTrue(response.getJSONArray("headers").toString().contains("Retry-After", ignoreCase = true))
            } finally { executor.shutdownNow() }
        }
    }

    @Test fun rejectsOriginsBeforeNetworking() {
        val executor = Executors.newSingleThreadExecutor()
        try {
            val done = CountDownLatch(1)
            val result = AtomicReference<JSONObject>()
            HttpTransport(setOf("https://backend.example"), executor)
                .execute(request("https://backend.example.attacker.test/api")) {
                    result.set(it); done.countDown()
                }
            assertTrue(done.await(5, TimeUnit.SECONDS))
            assertTrue(result.get().getJSONObject("Err").has("Url"))
        } finally { executor.shutdownNow() }
    }
}
