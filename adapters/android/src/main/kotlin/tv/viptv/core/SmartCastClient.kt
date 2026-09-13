package tv.viptv.core

import java.util.concurrent.Executor
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicReference
import org.json.JSONObject
import uniffi.viptv_core.SmartCastBridge

/** Android-mobile facade. UI receives only the final typed JSON result. */
class SmartCastClient(
    private val tvOrigin: String,
    deviceId: String,
    deviceName: String,
    private val tokenStore: SmartCastTokenStore,
    executor: Executor,
) : AutoCloseable {
    private val bridge = SmartCastBridge(
        JSONObject()
            .put("host", tvOrigin)
            .put("deviceId", deviceId)
            .put("deviceName", deviceName)
            .putOpt("authToken", tokenStore.load(tvOrigin))
            .toString(),
    )
    private val transport = SmartCastTransport(tvOrigin, executor)
    private val active = AtomicReference<SmartCastTransport.Call?>()
    private val running = AtomicBoolean(false)

    @Synchronized
    fun run(operation: String, input: JSONObject = JSONObject(), complete: (JSONObject) -> Unit) {
        check(running.compareAndSet(false, true)) { "A SmartCast command is already running" }
        try {
            drive(JSONObject(bridge.start(operation, input.toString())), complete)
        } catch (error: Exception) {
            running.set(false)
            throw error
        }
    }

    private fun drive(output: JSONObject, complete: (JSONObject) -> Unit) {
        when (output.getString("kind")) {
            "request" -> {
                val requestId = output.getLong("requestId").toUInt()
                transport.execute(output.getJSONObject("request"), active::set) { response ->
                    val next = response.fold(
                        onSuccess = { JSONObject(bridge.resolve(requestId, it.status.toUShort(), it.body)) },
                        onFailure = { JSONObject(bridge.reject(requestId)) },
                    )
                    active.set(null)
                    drive(next, complete)
                }
            }
            "complete" -> {
                if (output.optBoolean("credentialChanged")) {
                    bridge.credential()?.let { tokenStore.save(tvOrigin, it) }
                }
                running.set(false)
                complete(output)
            }
            else -> {
                running.set(false)
                complete(output)
            }
        }
    }

    @Synchronized
    fun forgetPairing() {
        active.getAndSet(null)?.cancel()
        running.set(false)
        bridge.cancel()
        bridge.clearCredential()
        tokenStore.clear(tvOrigin)
    }

    override fun close() {
        active.getAndSet(null)?.cancel()
        running.set(false)
        bridge.cancel()
        bridge.close()
    }
}
