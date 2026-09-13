package tv.viptv.core

import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference
import org.json.JSONObject
import org.junit.Assert.assertTrue
import org.junit.Test

class SmartCastTransportTest {
    @Test fun rejectsOffOriginRequestBeforeNetworking() {
        val executor = Executors.newSingleThreadExecutor()
        try {
            val completed = CountDownLatch(1)
            val result = AtomicReference<Result<SmartCastTransport.Response>>()
            val request = JSONObject()
                .put("method", "GET")
                .put("url", "https://192.0.2.11:7345/state/device/deviceinfo")
                .put("headers", JSONObject())
                .put("timeoutMillis", 1_000)
                .put("maxResponseBytes", 1_024)
            SmartCastTransport("https://192.0.2.10:7345", executor).execute(request) {
                result.set(it)
                completed.countDown()
            }
            assertTrue(completed.await(2, TimeUnit.SECONDS))
            assertTrue(result.get().isFailure)
        } finally {
            executor.shutdownNow()
        }
    }
}
