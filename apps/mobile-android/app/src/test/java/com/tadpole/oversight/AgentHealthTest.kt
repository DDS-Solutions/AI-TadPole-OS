package com.tadpole.oversight

import com.tadpole.oversight.ui.health.AgentHealthStatus
import com.tadpole.oversight.ui.health.AgentStatus
import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test

class AgentHealthTest {

    @Test
    fun testAgentHealthStatusDeserialization() {
        val jsonInput = """
            [
                {
                    "id": "ag-1",
                    "name": "Master Swarm Router",
                    "status": "RUNNING",
                    "stepCount": 142,
                    "activeTask": "Monitoring IPC channels & zero-trust bridge"
                },
                {
                    "id": "ag-3",
                    "name": "Vector RAG Indexer",
                    "status": "IDLE",
                    "stepCount": 450,
                    "activeTask": "Awaiting query embedding"
                },
                {
                    "id": "ag-5",
                    "name": "Panic Agent",
                    "status": "HALTED",
                    "stepCount": 0,
                    "activeTask": "Halted by emergency kill switch"
                }
            ]
        """.trimIndent()

        val json = Json { ignoreUnknownKeys = true }
        val agents = json.decodeFromString<List<AgentHealthStatus>>(jsonInput)

        assertEquals(3, agents.size)

        val first = agents[0]
        assertEquals("ag-1", first.id)
        assertEquals("Master Swarm Router", first.name)
        assertEquals(AgentStatus.RUNNING, first.status)
        assertEquals(142, first.stepCount)
        assertEquals("Monitoring IPC channels & zero-trust bridge", first.activeTask)

        val second = agents[1]
        assertEquals(AgentStatus.IDLE, second.status)

        val third = agents[2]
        assertEquals(AgentStatus.HALTED, third.status)
    }

    @Test
    fun testAgentHealthStatusModelCreation() {
        val agent = AgentHealthStatus(
            id = "ag-test",
            name = "Test Agent",
            status = AgentStatus.ERROR,
            stepCount = 99,
            activeTask = "Recovering from error state"
        )

        assertNotNull(agent)
        assertEquals("ag-test", agent.id)
        assertEquals(AgentStatus.ERROR, agent.status)
    }
}
