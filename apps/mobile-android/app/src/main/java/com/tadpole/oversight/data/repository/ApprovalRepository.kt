package com.tadpole.oversight.data.repository

import com.tadpole.oversight.data.remote.RemoteApiClient
import com.tadpole.oversight.data.settings.SettingsRepository
import com.tadpole.oversight.ui.oversight.PendingApproval
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
        decidedBy: String,
        signature: String
    ): Boolean {
        val nodeIp = settingsRepository.getNodeIp()
        return apiClient.submitRemoteDecision(
            nodeIp = nodeIp,
            approvalId = approvalId,
            decision = decision,
            decidedBy = decidedBy,
            signature = signature
        )
    }
}
