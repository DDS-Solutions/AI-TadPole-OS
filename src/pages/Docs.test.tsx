/**
 * @file Docs.test.tsx
 * @description Suite for the System Documentation and Knowledge Base page.
 * @module Pages/Docs
 * @testedBehavior
 * - Knowledge Retrieval: Category expansion and document auto-selection.
 * - Search & Filter: Repository searching within the Knowledge Base.
 * - Documentation Manual: TOC generation and section navigation via markdown header parsing.
 * - Resilience: Graceful handling of documentation service failures.
 * @aiContext
 * - Mocks react-markdown to simplify testing of rendered documentation content.
 * - Mocks lucide-react icons used in the documentation navigation tree.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import Docs from './Docs';
import { tadpole_os_service } from '../services/tadpoleos_service';

// Mock Dependencies
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        get_knowledge_docs: vi.fn(),
        get_operations_manual: vi.fn(),
        get_knowledge_doc: vi.fn()
    }
}));

// Mock react-markdown to simplify testing DOM output
vi.mock('react-markdown', () => ({
    default: ({ children }: any) => <div data-testid="markdown-content">{children}</div>
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
    Search: () => <div data-testid="icon-search" />,
    ChevronRight: () => <div data-testid="icon-chevron-right" />,
    ChevronDown: () => <div data-testid="icon-chevron-down" />,
    Book: () => <div data-testid="icon-book" />,
    List: () => <div data-testid="icon-list" />,
    Layout: () => <div data-testid="icon-layout" />
}));

// Mock i18n to return the key/label directly for testing
vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            if (key === 'docs.manual_sections') return 'Manual Sections';
            if (key === 'docs.tab_knowledge') return 'Knowledge Base';
            if (key === 'docs.tab_manual') return 'Operations Manual';
            if (key === 'docs.search_placeholder') return 'Search repository...';
            if (key === 'docs.error_title') return 'Connection Failed';
            if (options?.message) return options.message;
            return key;
        }
    }
}));

describe('Docs Page', () => {
    const mockDocsList = [
        { category: 'Architecture', name: 'architecture-overview', title: 'Architecture Overview' },
        { category: 'Architecture', name: 'database-schema', title: 'Database Schema' },
        { category: 'API', name: 'rest-api', title: 'REST API' }
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        
        // Setup default successful responses
        (tadpole_os_service.get_knowledge_docs as any).mockResolvedValue(mockDocsList);
        (tadpole_os_service.get_knowledge_doc as any).mockResolvedValue('# Knowledge Doc Content');
        (tadpole_os_service.get_operations_manual as any).mockResolvedValue('# Operations Manual Content\n## First Section');
    });

    it('renders the initial layout with knowledge base tab active', async () => {
        await act(async () => {
             render(<Docs />);
        });

        // Tabs
        expect(screen.getByText('Knowledge Base')).toBeInTheDocument();
        expect(screen.getByText('Operations Manual')).toBeInTheDocument();

        // Search input (only in KB view)
        expect(screen.getByPlaceholderText('Search repository...')).toBeInTheDocument();

        // Categories should be rendered
        expect(screen.getByText('Architecture')).toBeInTheDocument();
        expect(screen.getByText('API')).toBeInTheDocument();
        
        // Specific docs should be rendered (because categories auto-expand)
        expect(screen.getByText('Architecture Overview')).toBeInTheDocument();
        expect(screen.getByText('Database Schema')).toBeInTheDocument();
    });

    it('fetches and auto-selects a document containing "architecture" on load', async () => {
        await act(async () => {
            render(<Docs />);
        });

        // The component auto-selects 'architecture-overview' based on the name include check
        expect(tadpole_os_service.get_knowledge_doc).toHaveBeenCalledWith('Architecture', 'architecture-overview');
        
        // Mocked markdown content contains the resolved value
        const content = await screen.findByTestId('markdown-content');
        expect(content).toHaveTextContent('# Knowledge Doc Content');
    });

    it('allows selecting a different document', async () => {
        await act(async () => {
            render(<Docs />);
        });

        // Click on the API doc
        await act(async () => {
            fireEvent.click(screen.getByText('REST API'));
        });

        expect(tadpole_os_service.get_knowledge_doc).toHaveBeenCalledWith('API', 'rest-api');
    });

    it('filters knowledge base items using search', async () => {
        await act(async () => {
            render(<Docs />);
        });

        const searchInput = screen.getByPlaceholderText('Search repository...');
        
        // Search for 'rest'
        await act(async () => {
            fireEvent.change(searchInput, { target: { value: 'rest' } });
        });

        // API category and REST API doc should remain
        expect(screen.getByText('API')).toBeInTheDocument();
        expect(screen.getByText('REST API')).toBeInTheDocument();
        
        // Architecture should be filtered out
        expect(screen.queryByText('Architecture Overview')).not.toBeInTheDocument();
    });

    it('toggles to Operations Manual tab and fetches manual content', async () => {
        await act(async () => {
            render(<Docs />);
        });

        // Search input is present in Knowledge Base
        expect(screen.getByPlaceholderText('Search repository...')).toBeInTheDocument();

        // Click Manual tab
        await act(async () => {
            fireEvent.click(screen.getByText('Operations Manual'));
        });

        // Search should be gone
        expect(screen.queryByPlaceholderText('Search repository...')).not.toBeInTheDocument();

        // Manual Sections Title should appear
        expect(screen.getByText('Manual Sections')).toBeInTheDocument();

        // Fetched manual content
        expect(tadpole_os_service.get_operations_manual).toHaveBeenCalled();
        const content = await screen.findByTestId('markdown-content');
        expect(content).toHaveTextContent('# Operations Manual Content');
    });

    it('generates TOC from markdown headers in Operations Manual', async () => {
        await act(async () => {
            render(<Docs />);
        });

        // Click Manual tab
        await act(async () => {
            fireEvent.click(screen.getByText('Operations Manual'));
        });

        // '## First Section' was returned by the mock, so 'First Section' should be in the TOC sidebar
        expect(screen.getByText('First Section')).toBeInTheDocument();
    });

    it('handles fetch failures gracefully', async () => {
        (tadpole_os_service.get_knowledge_doc as any).mockRejectedValue(new Error('Network error 500'));
        
        await act(async () => {
            render(<Docs />);
        });

        const content = await screen.findByTestId('markdown-content');
        
        // Look for the error screen fallback defined in the catch block
        expect(content.textContent).toContain('Connection Failed');
        expect(content.textContent).toContain('Network error 500');
    });
});
