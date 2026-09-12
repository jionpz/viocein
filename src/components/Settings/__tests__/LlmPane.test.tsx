import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, waitFor, cleanup, within } from '@testing-library/react'
import { LlmPane } from '../LlmPane'
import * as tauri from '../../../lib/tauri'

vi.mock('../../../lib/tauri')

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        'settings.provider': 'Provider',
        'settings.apiKey': 'API Key',
        'settings.model': 'Model',
        'settings.baseUrl': 'Base URL',
        'settings.test': 'Test',
        'settings.enterApiKey': 'Enter API Key',
        'settings.connectionSuccess': 'Connection successful',
        'settings.connectionFailed': 'Connection failed',
        'settings.storedLocally': 'Stored locally',
        'settings.fetchModels': 'Fetch models',
        'settings.modelsAvailable': `${params?.count ?? 0} models available`,
        'settings.credentialSaveFailed': `Credential save failed: ${params?.details ?? ''}`,
        'settings.llmModelPlaceholder': 'e.g. gpt-4o-mini',
        'settings.enableAiPolish': 'AI cleanup for dictation',
        'settings.enableAiPolishDesc': 'Cleans up dictation before output',
        'settings.contextAdaptation': 'Adapt writing to the current app',
        'settings.contextAdaptationHint': 'Uses a private local app category',
        'settings.contextAdaptationApps': 'Apps adapted by context',
        'settings.lastDictationContext': 'Last dictation context',
        'settings.polishStyle': 'Polish style',
        'settings.polishStyleMinimal': 'Minimal',
        'settings.polishStyleClean': 'Clean',
        'settings.polishStyleStructured': 'Structured',
        'settings.polishStyleProfessional': 'Professional',
        'settings.advancedPolishSettings': 'Advanced polish settings',
        'settings.customPolishInstructions': 'Custom polish instructions',
        'settings.customPolishInstructionsPlaceholder': 'Example prompt',
        'settings.customPolishInstructionsCount': `${params?.count ?? 0} / 2000 characters`,
        'settings.translationMode': 'Always translate output',
        'settings.selectedTextContext': 'Use selected text as context',
        'settings.selectedTextContextDesc': 'Use selected text for context',
        'providers.llm.company': 'Company LLM Gateway',
        'providers.llm.ollama': 'Ollama (Local)',
      }
      return translations[key] ?? key
    },
  }),
}))

const mockAppStore = {
  config: {
    llm_provider: 'company' as string,
    llm_api_key: '',
    llm_base_url: 'https://llm.example.internal/v1',
    llm_model: 'company-model',
    polish_enabled: true,
    context_adaptation_enabled: true,
    polish_style: 'clean',
    polish_custom_prompt: '',
    polish_chinese_script: 'preserve',
    custom_scenes: [],
    active_scene: null as unknown,
    family_scene_assignments: [],
    translate_enabled: false,
    selected_text_enabled: false,
    target_lang: 'en',
    translation: { targets: ['en'], active_target: 'en' },
  },
  updateConfig: vi.fn(),
  llmTestStatus: 'idle' as 'idle' | 'testing' | 'success' | 'error',
  setLlmTestStatus: vi.fn(),
  llmLatencyMs: null as number | null,
  setLlmLatencyMs: vi.fn(),
  llmModels: [] as string[],
  setLlmModels: vi.fn(),
  lastContext: null as unknown,
}

vi.mock('../../../stores/appStore', () => ({
  useAppStore: (selector: (state: typeof mockAppStore) => unknown) =>
    typeof selector === 'function' ? selector(mockAppStore) : mockAppStore,
}))

function resetStore() {
  mockAppStore.config = {
    llm_provider: 'company',
    llm_api_key: '',
    llm_base_url: 'https://llm.example.internal/v1',
    llm_model: 'company-model',
    polish_enabled: true,
    context_adaptation_enabled: true,
    polish_style: 'clean',
    polish_custom_prompt: '',
    polish_chinese_script: 'preserve',
    custom_scenes: [],
    active_scene: null,
    family_scene_assignments: [],
    translate_enabled: false,
    selected_text_enabled: false,
    target_lang: 'en',
    translation: { targets: ['en'], active_target: 'en' },
  }
  mockAppStore.llmTestStatus = 'idle'
  mockAppStore.llmLatencyMs = null
  mockAppStore.llmModels = []
  mockAppStore.lastContext = null
}

describe('LlmPane (internal local-only build)', () => {
  beforeEach(() => {
    resetStore()
    vi.clearAllMocks()
    vi.mocked(tauri.readCredential).mockResolvedValue(null)
    vi.mocked(tauri.setCredential).mockResolvedValue(undefined)
    vi.mocked(tauri.fetchLlmModels).mockResolvedValue([])
    vi.mocked(tauri.benchLlmConnection).mockResolvedValue(42)
    vi.mocked(tauri.getLatestMappingCandidate).mockResolvedValue(null)
    vi.mocked(tauri.listCustomAppMappings).mockResolvedValue([])
  })

  afterEach(() => {
    cleanup()
    vi.clearAllMocks()
  })

  it('only offers the company gateway and local Ollama', () => {
    render(<LlmPane />)
    const providerSelect = screen.getAllByRole('combobox')[0]
    expect(within(providerSelect).getByRole('option', { name: 'Company LLM Gateway' })).toHaveValue(
      'company',
    )
    expect(within(providerSelect).getByRole('option', { name: 'Ollama (Local)' })).toHaveValue(
      'ollama',
    )
    expect(within(providerSelect).getAllByRole('option')).toHaveLength(2)
  })

  it('lets the operator edit the company gateway base URL', async () => {
    render(<LlmPane />)
    const baseUrl = screen.getByDisplayValue('https://llm.example.internal/v1')
    expect(baseUrl).not.toHaveAttribute('readonly')
    fireEvent.change(baseUrl, { target: { value: 'https://llm.corp.example/openai/v1' } })
    expect(mockAppStore.updateConfig).toHaveBeenCalledWith({
      llm_base_url: 'https://llm.corp.example/openai/v1',
    })
  })

  it('allows a keyless company gateway: the test button is not gated on an API key', () => {
    render(<LlmPane />)
    const testButton = screen.getByRole('button', { name: /Test/i })
    expect(testButton).toBeEnabled()
  })

  it('benches the company connection when the test button is pressed', async () => {
    render(<LlmPane />)
    fireEvent.click(screen.getByRole('button', { name: /Test/i }))
    await waitFor(() => {
      expect(tauri.benchLlmConnection).toHaveBeenCalledWith(
        '',
        'company',
        'https://llm.example.internal/v1',
        'company-model',
      )
    })
  })

  it('stores an optional company API key in the credential vault', async () => {
    render(<LlmPane />)
    fireEvent.change(screen.getByPlaceholderText('Enter API Key'), {
      target: { value: 'company-secret' },
    })
    await waitFor(
      () => {
        expect(tauri.setCredential).toHaveBeenCalledWith('llm', 'company', 'company-secret')
      },
      { timeout: 2000 },
    )
  })

  it('does not render an API key input for local Ollama', async () => {
    mockAppStore.config = { ...mockAppStore.config, llm_provider: 'ollama' }
    render(<LlmPane />)
    await waitFor(() => {
      expect(screen.queryByPlaceholderText('Enter API Key')).not.toBeInTheDocument()
    })
  })

  it('lets the user edit the Ollama base URL', async () => {
    mockAppStore.config = {
      ...mockAppStore.config,
      llm_provider: 'ollama',
      llm_base_url: 'http://localhost:11434/v1',
    }
    render(<LlmPane />)
    const baseUrl = await screen.findByDisplayValue('http://localhost:11434/v1')
    expect(baseUrl).not.toHaveAttribute('readonly')
    fireEvent.change(baseUrl, { target: { value: 'http://127.0.0.1:11434/v1' } })
    expect(mockAppStore.updateConfig).toHaveBeenCalledWith({
      llm_base_url: 'http://127.0.0.1:11434/v1',
    })
  })

  it('updates the provider and clears stale test state when switching', () => {
    render(<LlmPane />)
    fireEvent.change(screen.getAllByRole('combobox')[0], { target: { value: 'ollama' } })
    expect(mockAppStore.updateConfig).toHaveBeenCalledWith(
      expect.objectContaining({ llm_provider: 'ollama' }),
    )
    expect(mockAppStore.setLlmTestStatus).toHaveBeenCalledWith('idle')
  })
})
