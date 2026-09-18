/**
 * @docs ARCHITECTURE:Governance
 * ### AI Assist Note
 * Bridges signed companion decisions to the authoritative oversight API.
 * ### 🔍 Debugging & Observability
 * Failure Path: Signed decision rejection or pending-ledger fetch failure.
 * Telemetry Link: Search `Remote Oversight` in server traces.
 */
package com.tadpole.oversight.data.repository

import com.tadpole.oversight.data.remote.RemoteApiClient
import com.tadpole.oversight.data.settings.SettingsRepository
import com.tadpole.oversight.ui.oversight.PendingApproval
import com.tadpole.oversight.security.SignedDecisionProof
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

class ApprovalRepository(
    private val apiClient: RemoteApiClient,
    private val settingsRepository: SettingsRepository
) {

    fun getPendingApprovals(): Flow<List<PendingApproval>> = flow {
        val nodeIp = settingsRepository.getNodeIp()
        val items = apiClient.fetchPendingApprovals(nodeIp)
        emit(items)
    }

    suspend fun submitDecision(
        approvalId: String,
        decision: String,
        proof: SignedDecisionProof
    ): Boolean {
        val nodeIp = settingsRepository.getNodeIp()
        return apiClient.submitRemoteDecision(
            nodeIp = nodeIp,
            approvalId = approvalId,
            decision = decision,
            proof = proof
        )
    }
}
