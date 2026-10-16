/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Chat / Chat_Message_List Tests
 * - **Primary Entrypoints**: none declared
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Tests message list rendering and Socratic envelope card integration.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Chat_Message_List } from './Chat_Message_List';
import type { Chat_Message } from '../../stores/sovereign_store';
import type { SocraticEnvelope } from '../../types';

describe('Chat_Message_List', () => {
    it('renders empty state when no messages are provided', () => {
        render(
            <Chat_Message_List
                messages={[]}
                max_rendered={50}
                active_scope="swarm"
                target_node="node-0"
            />
        );
        expect(screen.getByText(/READY_FOR_INPUT/i)).toBeInTheDocument();
    });

    it('renders text messages from both user and agents', () => {
        const messages: Chat_Message[] = [
            {
                id: 'msg-1',
                sender_id: '0',
                sender_name: 'Human Operator',
                text: 'Run the security audit',
                timestamp: Date.now(),
                scope: 'swarm',
            },
            {
                id: 'msg-2',
                sender_id: 'agent-1',
                sender_name: 'Sentinel Agent',
                text: 'Security audit completed with zero vulnerabilities',
                timestamp: Date.now(),
                scope: 'swarm',
            },
        ];

        render(
            <Chat_Message_List
                messages={messages}
                max_rendered={50}
                active_scope="swarm"
                target_node="node-0"
            />
        );

        expect(screen.getByText('Run the security audit')).toBeInTheDocument();
        expect(screen.getByText('Security audit completed with zero vulnerabilities')).toBeInTheDocument();
        expect(screen.getByText('Human Operator')).toBeInTheDocument();
        expect(screen.getByText('Sentinel Agent')).toBeInTheDocument();
    });

    it('renders Socratic Envelope card when part.type is socratic', () => {
        const envelope: SocraticEnvelope = {
            status: 'PRE_CLEARED_GATE_PASS',
            scope_contract: {
                mission_vector: 'Deep Security Audit',
                target_paths: ['src/', 'server-rs/'],
            },
            performance_threshold: {
                active_model_slot: 2,
                target_turn_latency_ms: 1500,
            },
            architecture_mode: {
                mode_name: 'Nexus Engineer Mode',
            },
            pre_cleared_failure_modes: [
                'Circuit Breaker (3 Consecutive Fails)',
                'Air-Gap Shield',
            ],
        };

        const messages: Chat_Message[] = [
            {
                id: 'msg-socratic-1',
                sender_id: 'agent-nexus',
                sender_name: 'Nexus Engineer',
                text: '',
                timestamp: Date.now(),
                scope: 'agent',
                parts: [
                    {
                        type: 'socratic',
                        envelope,
                    },
                ],
            },
        ];

        render(
            <Chat_Message_List
                messages={messages}
                max_rendered={50}
                active_scope="agent"
                target_node="agent-nexus"
            />
        );

        expect(screen.getByText('PRE_CLEARED_GATE_PASS')).toBeInTheDocument();
        expect(screen.getByText('Slot 2')).toBeInTheDocument();
        expect(screen.getByText('Nexus Engineer Mode')).toBeInTheDocument();
        expect(screen.getByText('Deep Security Audit')).toBeInTheDocument();
        expect(screen.getByText('1500ms')).toBeInTheDocument();
        expect(screen.getByText('Circuit Breaker (3 Consecutive Fails)')).toBeInTheDocument();
    });

    it('renders Socratic Envelope card when raw message text contains JSON gate pass', () => {
        const envelope: SocraticEnvelope = {
            status: 'PRE_CLEARED_GATE_PASS',
            scope_contract: {
                mission_vector: 'Infrastructure Refactor',
            },
            performance_threshold: {
                active_model_slot: 3,
                target_turn_latency_ms: 10000,
            },
            architecture_mode: {
                mode_name: 'Sovereign 3-Layer Architecture',
            },
            pre_cleared_failure_modes: ['Deterministic Execution Sandbox'],
        };

        const messages: Chat_Message[] = [
            {
                id: 'msg-socratic-json',
                sender_id: 'agent-router',
                sender_name: 'Router Agent',
                text: JSON.stringify(envelope),
                timestamp: Date.now(),
                scope: 'swarm',
            },
        ];

        render(
            <Chat_Message_List
                messages={messages}
                max_rendered={50}
                active_scope="swarm"
                target_node="node-0"
            />
        );

        expect(screen.getByText('PRE_CLEARED_GATE_PASS')).toBeInTheDocument();
        expect(screen.getByText('Slot 3')).toBeInTheDocument();
        expect(screen.getByText('Sovereign 3-Layer Architecture')).toBeInTheDocument();
        expect(screen.getByText('Infrastructure Refactor')).toBeInTheDocument();
        expect(screen.getByText('10000ms')).toBeInTheDocument();
        expect(screen.getByText('Deterministic Execution Sandbox')).toBeInTheDocument();
    });
});
