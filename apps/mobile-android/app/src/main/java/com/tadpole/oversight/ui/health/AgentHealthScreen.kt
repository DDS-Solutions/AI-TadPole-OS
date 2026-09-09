/**
 * @docs ARCHITECTURE:UI
 * ### AI Assist Note
 * Signed companion health monitor and emergency freeze control.
 * ### 🔍 Debugging & Observability
 * Failure Path: Signed health fetch or emergency freeze rejection.
 * Telemetry Link: Observe the companion snackbar and server security traces.
 */
package com.tadpole.oversight.ui.health

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MonitorHeart
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.tadpole.oversight.data.remote.RemoteApiClient
import com.tadpole.oversight.data.settings.SettingsRepository
import com.tadpole.oversight.security.BiometricSignatureManager
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable

@Serializable
enum class AgentStatus {
    RUNNING, IDLE, HALTED, ERROR
}

@Serializable
data class AgentHealthStatus(
    val id: String,
    val name: String,
    val status: AgentStatus,
    val stepCount: Int,
    val activeTask: String
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AgentHealthScreen() {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val snackbarHostState = remember { SnackbarHostState() }
    val settingsRepository = remember { SettingsRepository(context) }
    val signatureManager = remember { BiometricSignatureManager(context) }
    val deviceId = remember { settingsRepository.getOrCreateDeviceId() }
    val apiClient = remember { RemoteApiClient(deviceId, signatureManager::signPayload) }

    var showPanicDialog by remember { mutableStateOf(false) }
    var isLoading by remember { mutableStateOf(true) }
    var sampleAgents by remember { mutableStateOf(emptyList<AgentHealthStatus>()) }

    LaunchedEffect(Unit) {
        while (isActive) {
            val liveAgents = apiClient.fetchAgentHealth(settingsRepository.getNodeIp())
            sampleAgents = liveAgents
            isLoading = false
            kotlinx.coroutines.delay(3000)
        }
    }

    fun handleEmergencyFreezeAll() {
        scope.launch {
            val success = apiClient.triggerEmergencyFreeze(settingsRepository.getNodeIp())
            if (success) {
                sampleAgents = sampleAgents.map {
                    it.copy(status = AgentStatus.HALTED, activeTask = "SYSTEM PANIC FREEZE ENFORCED")
                }
                snackbarHostState.showSnackbar("EMERGENCY PANIC FREEZE TRANSMITTED TO SWARM")
            } else {
                snackbarHostState.showSnackbar("FAILED TO TRANSMIT EMERGENCY FREEZE - CHECK PAIRING")
            }
        }
    }

    Scaffold(snackbarHost = { SnackbarHost(hostState = snackbarHostState) }) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .padding(paddingValues)
                .padding(16.dp)
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.padding(bottom = 16.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.MonitorHeart,
                    contentDescription = "Swarm Heartbeat Monitor Icon",
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(28.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    text = "Agent Swarm Health",
                    style = MaterialTheme.typography.headlineSmall.copy(
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onBackground
                    )
                )
            }

            Button(
                onClick = { showPanicDialog = true },
                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error),
                shape = RoundedCornerShape(8.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .defaultMinSize(minHeight = 48.dp)
                    .padding(bottom = 16.dp)
            ) {
                Icon(
                    Icons.Default.Warning,
                    contentDescription = "Emergency Panic Warning",
                    modifier = Modifier.size(20.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    text = "EMERGENCY FREEZE ALL SWARMS",
                    fontWeight = FontWeight.Bold,
                    fontSize = 13.sp
                )
            }

            if (showPanicDialog) {
                AlertDialog(
                    onDismissRequest = { showPanicDialog = false },
                    icon = {
                        Icon(
                            Icons.Default.Warning,
                            contentDescription = "Warning Alert",
                            tint = MaterialTheme.colorScheme.error
                        )
                    },
                    title = { Text("Confirm Emergency Swarm Freeze") },
                    text = {
                        Text("Are you sure you want to halt all active agent swarms? This will immediately pause all execution pipelines across all nodes.")
                    },
                    confirmButton = {
                        Button(
                            onClick = {
                                showPanicDialog = false
                                handleEmergencyFreezeAll()
                            },
                            colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error),
                            modifier = Modifier.defaultMinSize(minHeight = 48.dp)
                        ) {
                            Text("Confirm Freeze")
                        }
                    },
                    dismissButton = {
                        OutlinedButton(
                            onClick = { showPanicDialog = false },
                            modifier = Modifier.defaultMinSize(minHeight = 48.dp)
                        ) {
                            Text("Cancel")
                        }
                    }
                )
            }

            if (isLoading) {
                Box(
                    modifier = Modifier.fillMaxWidth().padding(32.dp),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator(color = MaterialTheme.colorScheme.primary)
                }
            } else if (sampleAgents.isEmpty()) {
                Card(
                    shape = RoundedCornerShape(12.dp),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant),
                    modifier = Modifier.fillMaxWidth().padding(vertical = 16.dp)
                ) {
                    Column(
                        modifier = Modifier.padding(24.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Text(
                            text = "No Live Agents Found",
                            style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "Ensure the server is running and this companion remains paired in desktop settings.",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            } else {
                LazyVerticalGrid(
                    columns = GridCells.Adaptive(minSize = 150.dp),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    items(sampleAgents, key = { it.id }) { agent ->
                        AgentHealthCard(agent = agent)
                    }
                }
            }
        }
    }
}

@Composable
fun AgentHealthCard(agent: AgentHealthStatus) {
    val statusColor = when (agent.status) {
        AgentStatus.RUNNING -> MaterialTheme.colorScheme.primary
        AgentStatus.IDLE -> MaterialTheme.colorScheme.secondary
        AgentStatus.HALTED, AgentStatus.ERROR -> MaterialTheme.colorScheme.error
    }

    Card(
        shape = RoundedCornerShape(12.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Row(
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(
                    text = agent.name,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurface,
                    fontSize = 14.sp
                )
                Surface(
                    color = statusColor.copy(alpha = 0.2f),
                    shape = RoundedCornerShape(4.dp)
                ) {
                    Text(
                        text = agent.status.name,
                        color = statusColor,
                        fontFamily = FontFamily.Monospace,
                        fontWeight = FontWeight.Bold,
                        fontSize = 12.sp,
                        modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                    )
                }
            }
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = "Steps Executed: ${agent.stepCount}",
                fontFamily = FontFamily.Monospace,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontSize = 12.sp
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = "Task: ${agent.activeTask}",
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontSize = 12.sp
            )
        }
    }
}
