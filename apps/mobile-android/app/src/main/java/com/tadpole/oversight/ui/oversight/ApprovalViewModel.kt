/**
 * @docs ARCHITECTURE:Governance
 * ### AI Assist Note
 * Coordinates remote oversight polling and server-confirmed decisions.
 * ### 🔍 Debugging & Observability
 * Failure Path: Polling interruption or rejected signed decision.
 * Telemetry Link: Observe messageState and remote server traces.
 */
package com.tadpole.oversight.ui.oversight

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.tadpole.oversight.data.repository.ApprovalRepository
import com.tadpole.oversight.security.SignedDecisionProof
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

class ApprovalViewModel(
    private val repository: ApprovalRepository
) : ViewModel() {

    private val _approvalsState = MutableStateFlow<List<PendingApproval>>(emptyList())
    val approvalsState: StateFlow<List<PendingApproval>> = _approvalsState.asStateFlow()

    private val _messageState = MutableStateFlow<String?>(null)
    val messageState: StateFlow<String?> = _messageState.asStateFlow()

    init {
        startPollingApprovals()
    }

    fun loadApprovals() {
        viewModelScope.launch {
            repository.getPendingApprovals().collect { items ->
                _approvalsState.value = items
            }
        }
    }

    fun clearMessage() {
        _messageState.value = null
    }

    private fun startPollingApprovals() {
        viewModelScope.launch {
            while (isActive) {
                try {
                    repository.getPendingApprovals().collect { items ->
                        _approvalsState.value = items
                    }
                } catch (e: Exception) {
                    // Ignore connection hiccups
                }
                delay(3000)
            }
        }
    }

    fun decideItem(approval: PendingApproval, decision: String, proof: SignedDecisionProof) {
        viewModelScope.launch {
            val success = repository.submitDecision(
                approvalId = approval.id,
                decision = decision,
                proof = proof
            )
            if (success) {
                _approvalsState.update { list -> list.filter { it.id != approval.id } }
                _messageState.value = "Signed & ${decision.replaceFirstChar { it.uppercase() }}: ${approval.id}"
            } else {
                _messageState.value = "Server rejected the signed decision for ${approval.id}"
            }
        }
    }
}
