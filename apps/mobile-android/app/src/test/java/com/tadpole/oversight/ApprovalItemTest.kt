package com.tadpole.oversight

import com.tadpole.oversight.ui.oversight.PendingApproval
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test

class ApprovalItemTest {

    @Test
    fun testPendingApprovalDataModel() {
        val approval = PendingApproval(
            id = "ovr-100",
            agentName = "TestAgent",
            toolName = "execute_command",
            targetResource = "cargo check",
            rationale = "Testing approval pipeline",
            timestamp = "12:00:00"
        )

        assertNotNull(approval)
        assertEquals("ovr-100", approval.id)
        assertEquals("TestAgent", approval.agentName)
        assertEquals("execute_command", approval.toolName)
    }
}
