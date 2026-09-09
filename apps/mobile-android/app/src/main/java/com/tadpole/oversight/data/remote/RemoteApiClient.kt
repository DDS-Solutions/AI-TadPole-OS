/**
 * @docs ARCHITECTURE:Networking
 * ### AI Assist Note
 * Signed Android companion transport for remote pairing and oversight.
 * ### 🔍 Debugging & Observability
 * Failure Path: Pairing rejection, stale proof, or unreachable engine.
 * Telemetry Link: Search `TadpoleRemote` in Android logs.
 */
package com.tadpole.oversight.data.remote

import com.tadpole.oversight.ui.health.AgentHealthStatus
import com.tadpole.oversight.ui.health.AgentStatus
import com.tadpole.oversight.ui.oversight.PendingApproval
import com.tadpole.oversight.security.SignedDecisionProof
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.engine.okhttp.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.plugins.websocket.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import java.util.UUID

data class PingResult(
    val isSuccess: Boolean,
    val resolvedAddress: String,
    val message: String,
    val isMdnsFallback: Boolean = false
)

class RemoteApiClient(
    private val deviceId: String? = null,
    private val payloadSigner: ((String) -> String)? = null
) {

    private val client = HttpClient(OkHttp) {
        install(ContentNegotiation) {
            json(Json {
                ignoreUnknownKeys = true
                prettyPrint = true
            })
        }
        install(WebSockets)
        engine {
            config {
                connectTimeout(3, java.util.concurrent.TimeUnit.SECONDS)
                readTimeout(3, java.util.concurrent.TimeUnit.SECONDS)
                writeTimeout(3, java.util.concurrent.TimeUnit.SECONDS)
            }
        }
    }

    suspend fun pingNode(nodeIp: String): Boolean {
        return try {
            val formatted = if (nodeIp.contains(":")) nodeIp else "$nodeIp:8000"
            val response = client.get("http://$formatted/v1/remote/ping")
            response.status.isSuccess()
        } catch (e: Exception) {
            false
        }
    }

    suspend fun pingNodeDetailed(primaryAddress: String): PingResult {
        val target = primaryAddress.trim().ifBlank { "10.0.0.1:8000" }
        val formattedTarget = if (target.contains(":")) target else "$target:8000"

        // 1. Check primary specified node address
        if (pingNode(formattedTarget)) {
            return PingResult(
                isSuccess = true,
                resolvedAddress = formattedTarget,
                message = "🟢 Local Company Network ($formattedTarget) is ONLINE & REACHABLE!"
            )
        }

        // 2. Local network Wi-Fi (192.168.1.x, 192.168.50.x, 192.168.101.x) & mDNS fallback discovery
        val mdnsCandidates = listOf(
            "10.0.0.1:8000",
            "10.0.0.1:8000",
            "10.0.0.1:8000",
            "10.0.0.1:8000",
            "10.0.0.1:8000",
            "10.0.0.1:8000",
            "10.0.0.1:8000",
            "10.0.0.1:8000",
            "tadpole.local:8000",
            "tadpole-os.local:8000",
            "localhost:8000",
            "127.0.0.1:8000"
        ).filter { it != formattedTarget }

        for (candidate in mdnsCandidates) {
            if (pingNode(candidate)) {
                return PingResult(
                    isSuccess = true,
                    resolvedAddress = candidate,
                    message = "🟢 Local Network Node ($candidate) DISCOVERED & REACHABLE!",
                    isMdnsFallback = true
                )
            }
        }

        return PingResult(
            isSuccess = false,
            resolvedAddress = formattedTarget,
            message = "🔴 Unable to reach $formattedTarget or LAN (10.0.0.1:8000). Ensure server-rs engine is running!"
        )
    }

    private fun formatIp(nodeIp: String): String {
        val target = nodeIp.trim().ifBlank { "10.0.0.1:8000" }
        return if (target.contains(":")) target else "$target:8000"
    }

    private fun HttpRequestBuilder.applySignedProof(method: HttpMethod, path: String) {
        val pairedDeviceId = requireNotNull(deviceId) { "Companion device is not paired" }
        val signer = requireNotNull(payloadSigner) { "Companion signing identity is unavailable" }
        val timestamp = System.currentTimeMillis() / 1000
        val nonce = UUID.randomUUID().toString().replace("-", "")
        val canonical = "${method.value.uppercase()}:$path:$timestamp:$nonce"
        header("X-Device-Id", pairedDeviceId)
        header("X-Timestamp", timestamp.toString())
        header("X-Nonce", nonce)
        header("X-Signature", signer(canonical))
    }

    suspend fun fetchPendingApprovals(nodeIp: String): List<PendingApproval> {
        return try {
            val formatted = formatIp(nodeIp)
            android.util.Log.d("TadpoleRemote", "Fetching pending approvals from http://$formatted/v1/remote/oversight/pending")
            val path = "/v1/remote/oversight/pending"
            val response = client.get("http://$formatted$path") {
                applySignedProof(HttpMethod.Get, path)
            }
            if (response.status.isSuccess()) {
                val bodyText = response.bodyAsText()
                val json = Json { ignoreUnknownKeys = true }
                val items = json.decodeFromString<List<PendingApproval>>(bodyText)
                android.util.Log.d("TadpoleRemote", "Successfully fetched ${items.size} pending approvals from server")
                items
            } else {
                android.util.Log.w("TadpoleRemote", "Server returned non-success status: ${response.status}")
                defaultSampleApprovals()
            }
        } catch (e: Exception) {
            android.util.Log.e("TadpoleRemote", "Failed to fetch pending approvals: ${e.message}", e)
            defaultSampleApprovals()
        }
    }

    suspend fun fetchAgentHealth(nodeIp: String): List<AgentHealthStatus> {
        return try {
            val formatted = formatIp(nodeIp)
            android.util.Log.d("TadpoleRemote", "Fetching agent health from http://$formatted/v1/remote/agents/health")
            val path = "/v1/remote/agents/health"
            val response = client.get("http://$formatted$path") {
                applySignedProof(HttpMethod.Get, path)
            }
            if (response.status.isSuccess()) {
                val bodyText = response.bodyAsText()
                val json = Json { ignoreUnknownKeys = true }
                val agents = json.decodeFromString<List<AgentHealthStatus>>(bodyText)
                android.util.Log.d("TadpoleRemote", "Successfully fetched ${agents.size} agent health items from server")
                agents
            } else {
                android.util.Log.w("TadpoleRemote", "Server returned non-success status: ${response.status}")
                emptyList()
            }
        } catch (e: Exception) {
            android.util.Log.e("TadpoleRemote", "Failed to fetch agent health: ${e.message}", e)
            emptyList()
        }
    }

    private fun defaultSampleApprovals(): List<PendingApproval> {
        return emptyList()
    }

    suspend fun pairWithNode(
        nodeIp: String,
        pairingToken: String,
        deviceId: String,
        deviceName: String,
        userName: String,
        publicKey: String
    ): Boolean {
        return try {
            val formatted = formatIp(nodeIp)
            val response = client.post("http://$formatted/v1/remote/pair") {
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject {
                    put("token", pairingToken)
                    put("device_id", deviceId)
                    put("device_name", deviceName)
                    put("user_name", userName)
                    put("public_key", publicKey)
                })
            }
            response.status == HttpStatusCode.Created || response.status == HttpStatusCode.OK
        } catch (e: Exception) {
            false
        }
    }

    suspend fun submitRemoteDecision(
        nodeIp: String,
        approvalId: String,
        decision: String,
        proof: SignedDecisionProof
    ): Boolean {
        return try {
            val formatted = formatIp(nodeIp)
            val pairedDeviceId = requireNotNull(deviceId) { "Companion device is not paired" }
            val response = client.post("http://$formatted/v1/remote/oversight/decide") {
                contentType(ContentType.Application.Json)
                header("X-Device-Id", pairedDeviceId)
                setBody(buildJsonObject {
                    put("approval_id", approvalId)
                    put("decision", decision)
                    put("decided_by", pairedDeviceId)
                    put("signature", proof.signature)
                    put("timestamp", proof.timestamp)
                    put("nonce", proof.nonce)
                })
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            false
        }
    }

    suspend fun triggerEmergencyFreeze(nodeIp: String): Boolean {
        return try {
            val formatted = formatIp(nodeIp)
            val path = "/v1/remote/agents/halt"
            val response = client.post("http://$formatted$path") {
                contentType(ContentType.Application.Json)
                applySignedProof(HttpMethod.Post, path)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            false
        }
    }

    fun observeTelemetryStream(nodeIp: String): Flow<String> = flow {
        emit("WS_CONNECTED")
    }
}
