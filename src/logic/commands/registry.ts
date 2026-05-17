/**
 * @docs ARCHITECTURE:Logic:Commands
 */

import type { Command_Definition, Command_Context, Command_Result } from './types';

class Command_Registry {
    private handlers: Map<string, Command_Definition> = new Map();

    public register(definition: Command_Definition) {
        this.handlers.set(definition.command.toLowerCase(), definition);
    }

    public async execute(cmd: string, ctx: Command_Context): Promise<Command_Result> {
        const handler = this.handlers.get(cmd.toLowerCase());
        if (handler) {
            return await handler.handler(ctx);
        }
        return { should_clear_logs: false, handled: false };
    }

    public get_definitions(): Command_Definition[] {
        return Array.from(this.handlers.values());
    }
}

export const command_registry = new Command_Registry();
