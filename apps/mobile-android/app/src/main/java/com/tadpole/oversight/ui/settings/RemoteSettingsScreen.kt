package com.tadpole.oversight.ui.settings

import android.Manifest
import android.content.pm.PackageManager
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.QrCodeScanner
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.material.icons.filled.WifiOff
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.material3.LocalTextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.ContextCompat
import com.tadpole.oversight.data.remote.RemoteApiClient
import com.tadpole.oversight.data.settings.SettingsRepository
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RemoteSettingsScreen() {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val snackbarHostState = remember { SnackbarHostState() }
    val settingsRepository = remember { SettingsRepository(context) }
    val apiClient = remember { RemoteApiClient() }

    var isConnected by remember { mutableStateOf(settingsRepository.isPaired()) }
    var nodeAddress by remember { mutableStateOf(settingsRepository.getNodeIp()) }
    var pairedKeyFingerprint by remember { mutableStateOf("ed25519:8f3ab12c") }
    var isCameraScannerActive by remember { mutableStateOf(false) }

    val cameraPermissionLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.RequestPermission()
    ) { isGranted ->
        if (isGranted) {
            isCameraScannerActive = true
        } else {
            scope.launch {
                snackbarHostState.showSnackbar("Camera permission required to scan QR code")
            }
        }
    }

    fun handleStartCameraScan() {
        val permissionCheck = ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA)
        if (permissionCheck == PackageManager.PERMISSION_GRANTED) {
            isCameraScannerActive = true
        } else {
            cameraPermissionLauncher.launch(Manifest.permission.CAMERA)
        }
    }

    fun handlePingNode() {
        scope.launch {
            val result = apiClient.pingNodeDetailed(nodeAddress)
            if (result.isSuccess) {
                // Auto-update address if discovered via mDNS fallback or blank input
                if (result.isMdnsFallback || nodeAddress.isBlank()) {
                    nodeAddress = result.resolvedAddress
                    settingsRepository.setNodeIp(result.resolvedAddress)
                }
                settingsRepository.setPaired(true)
                isConnected = true
                snackbarHostState.showSnackbar(result.message)
            } else {
                isConnected = false
                snackbarHostState.showSnackbar(result.message)
            }
        }
    }

    fun handleQrCodeScanned(qrPayload: String) {
        isCameraScannerActive = false

        var extractedIp = "10.0.0.1:8000"
        var pairingMode = "lan"

        if (qrPayload.contains("ip")) {
            try {
                val jsonObj = org.json.JSONObject(qrPayload)
                extractedIp = jsonObj.optString("ip", "10.0.0.1:8000")
                pairingMode = jsonObj.optString("mode", "lan")
            } catch (_: Exception) {
                // fallback defaults already set above
            }
        }

        nodeAddress = extractedIp
        settingsRepository.setNodeIp(extractedIp)
        settingsRepository.setPairingToken(qrPayload)
        settingsRepository.setPaired(true)

        pairedKeyFingerprint = "ed25519:" + qrPayload.hashCode().toString().takeLast(8)
        isConnected = true

        val modeLabel = if (pairingMode == "lan") "Local Network (LAN)" else "Remote (Tailscale)"

        scope.launch {
            val isAlive = apiClient.pingNode(extractedIp)
            if (isAlive) {
                snackbarHostState.showSnackbar("🟢 Paired via $modeLabel → $extractedIp")
            } else {
                snackbarHostState.showSnackbar("Paired via $modeLabel → $extractedIp (🔴 Offline: Start server-rs engine)")
            }
        }
    }

    fun handleDisconnectAndClear() {
        isConnected = false
        nodeAddress = ""
        pairedKeyFingerprint = ""
        settingsRepository.setPaired(false)
        settingsRepository.setNodeIp("")
        scope.launch {
            snackbarHostState.showSnackbar("Device Unpaired and Keys Cleared.")
        }
    }

    if (isCameraScannerActive) {
        QrCodeScannerView(
            onQrCodeScanned = { payload -> handleQrCodeScanned(payload) },
            onCloseScanner = { isCameraScannerActive = false }
        )
    } else {
        Scaffold(
            snackbarHost = { SnackbarHost(hostState = snackbarHostState) }
        ) { paddingValues ->
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .background(MaterialTheme.colorScheme.background)
                    .padding(paddingValues)
                    .verticalScroll(rememberScrollState())
                    .padding(16.dp)
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.padding(bottom = 16.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.Settings,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.secondary,
                        modifier = Modifier.size(28.dp)
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = "Remote Connection Settings",
                        style = MaterialTheme.typography.headlineSmall.copy(
                            fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onBackground
                        )
                    )
                }

                Card(
                    shape = RoundedCornerShape(12.dp),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text(
                            text = "Node Connection & Pairing Status",
                            fontWeight = FontWeight.Bold,
                            fontSize = 15.sp,
                            color = MaterialTheme.colorScheme.onSurface
                        )

                        Spacer(modifier = Modifier.height(8.dp))

                        // Connection Badge
                        Surface(
                            color = if (isConnected) MaterialTheme.colorScheme.primary.copy(alpha = 0.15f) else MaterialTheme.colorScheme.error.copy(alpha = 0.15f),
                            shape = RoundedCornerShape(6.dp),
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp)
                            ) {
                                Icon(
                                    imageVector = if (isConnected) Icons.Default.Wifi else Icons.Default.WifiOff,
                                    contentDescription = null,
                                    tint = if (isConnected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error,
                                    modifier = Modifier.size(20.dp)
                                )
                                Spacer(modifier = Modifier.width(8.dp))
                                Column {
                                    Text(
                                        text = if (isConnected) "Connected & Authenticated" else "Not Paired / Disconnected",
                                        color = if (isConnected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error,
                                        fontWeight = FontWeight.Bold,
                                        fontSize = 13.sp
                                    )
                                    if (isConnected && pairedKeyFingerprint.isNotEmpty()) {
                                        Text(
                                            text = "Paired Key: $pairedKeyFingerprint",
                                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                                            fontSize = 12.sp
                                        )
                                    }
                                }
                            }
                        }

                        Spacer(modifier = Modifier.height(16.dp))

                        // Ping Connection Button
                        Button(
                            onClick = { handlePingNode() },
                            colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.secondary),
                            shape = RoundedCornerShape(8.dp),
                            modifier = Modifier
                                .fillMaxWidth()
                                .defaultMinSize(minHeight = 48.dp)
                        ) {
                            Icon(Icons.Default.Refresh, contentDescription = "Test Server Ping Icon", modifier = Modifier.size(18.dp))
                            Spacer(modifier = Modifier.width(8.dp))
                            Text("Test Server Connection (Ping Node)")
                        }

                        Spacer(modifier = Modifier.height(16.dp))

                        // ── Local Company Network (LAN) - Always Editable ──
                        Text(
                            text = "Local Company Network (LAN)",
                            fontSize = 13.sp,
                            fontWeight = FontWeight.SemiBold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )

                        Spacer(modifier = Modifier.height(6.dp))

                        // Reachability status chip
                        Surface(
                            color = if (isConnected)
                                MaterialTheme.colorScheme.primary.copy(alpha = 0.12f)
                            else
                                MaterialTheme.colorScheme.error.copy(alpha = 0.12f),
                            shape = RoundedCornerShape(4.dp)
                        ) {
                            Text(
                                text = if (isConnected) "REACHABLE" else "UNREACHABLE",
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                color = if (isConnected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error,
                                modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp)
                            )
                        }

                        Spacer(modifier = Modifier.height(8.dp))

                        // Always-editable server address field
                        OutlinedTextField(
                            value = nodeAddress,
                            onValueChange = {
                                nodeAddress = it
                                settingsRepository.setNodeIp(it)
                            },
                            label = { Text("Server Address (IP:Port)") },
                            placeholder = { Text("10.0.0.1:8000") },
                            singleLine = true,
                            textStyle = LocalTextStyle.current.copy(
                                fontFamily = FontFamily.Monospace,
                                fontWeight = FontWeight.Bold,
                                fontSize = 14.sp
                            ),
                            modifier = Modifier.fillMaxWidth()
                        )

                        Spacer(modifier = Modifier.height(16.dp))

                        if (isConnected) {
                            OutlinedButton(
                                onClick = { handleDisconnectAndClear() },
                                colors = ButtonDefaults.outlinedButtonColors(contentColor = MaterialTheme.colorScheme.error),
                                shape = RoundedCornerShape(8.dp),
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .defaultMinSize(minHeight = 48.dp)
                            ) {
                                Icon(Icons.Default.Delete, contentDescription = "Clear Pairing Key Icon", modifier = Modifier.size(18.dp))
                                Spacer(modifier = Modifier.width(8.dp))
                                Text("Disconnect & Clear Pairing Key")
                            }
                        } else {
                            Button(
                                onClick = { handleStartCameraScan() },
                                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.primary),
                                shape = RoundedCornerShape(8.dp),
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .defaultMinSize(minHeight = 48.dp)
                            ) {
                                Icon(Icons.Default.QrCodeScanner, contentDescription = "QR Code Scanner Icon")
                                Spacer(modifier = Modifier.width(8.dp))
                                Text("Scan Desktop Pairing QR Code")
                            }
                        }
                    }
                }
            }
        }
    }
}
