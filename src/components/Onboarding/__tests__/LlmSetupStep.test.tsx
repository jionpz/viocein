import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { LlmSetupStep } from '../LlmSetupStep'
import { LLM_DEFAULT_CONFIG } from '../../../lib/constants'
import * as tauri from '../../../lib/tauri'

const mockStore = {
  config: {
    llm_provider: 'ollama',
    llm_api_key: '',
    llm_base_url: 'http://localhost:11434/v1',
    llm_model: 'llama3.2',
  },
  updateConfig: vi.fn(),
  llmTestStatus: 'idle',
  setLlmTestStatus: vi.fn(),
}

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) =>
      ({
        'onboarding.llm.serviceLabel': 'Service',
        'onboarding.llm.apiKeyLabel': 'API key',
        'onboarding.llm.apiKeyPlaceholder': 'API key',
        'onboarding.llm.testButton': 'Test',
        'onboarding.llm.modelLabel': 'Model',
        'onboarding.llm.modelPlaceholder': 'Model',
        'onboarding.llm.fetchModelsTitle': 'Fetch models',
        'onboarding.llm.baseUrlLabel': 'Base URL',
        'providers.llm.company': 'Company gateway',
        'providers.llm.ollama': 'Ollama',
      })[key] ?? key,
  }),
}))

vi.mock('../../../stores/appStore', () => ({
  useAppStore: (selector: any) => selector(mockStore),
}))

vi.mock('../../../lib/tauri')

beforeEach(() => {
  mockStore.config = {
    llm_provider: 'ollama',
    llm_api_key: '',
    llm_base_url: 'http://localhost:11434/v1',
    llm_model: 'llama3.2',
  }
  mockStore.updateConfig = vi.fn()
  mockStore.llmTestStatus = 'idle'
  mockStore.setLlmTestStatus = vi.fn()
  vi.clearAllMocks()
  vi.mocked(tauri.testLlmConnection).mockResolvedValue(true)
  vi.mocked(tauri.fetchLlmModels).mockResolvedValue(['llama3.2'])
})

afterEach(cleanup)

describe('LlmSetupStep (internal local-only build)', () => {
  it('offers only the company gateway and Ollama', () => {
    render(<LlmSetupStep />)

    const providerSelect = screen.getAllByRole('combobox')[0]
    const options = within(providerSelect).getAllByRole('option')
    expect(options).toHaveLength(2)
    expect(within(providerSelect).getByRole('option', { name: 'Company gateway' })).toHaveValue(
      'company',
    )
    expect(within(providerSelect).getByRole('option', { name: 'Ollama' })).toHaveValue('ollama')
  })

  it('migrates an unsupported saved provider to the company gateway', async () => {
    mockStore.config = {
      llm_provider: 'zhipu',
      llm_api_key: '',
      llm_base_url: 'https://open.bigmodel.cn/api/paas/v4',
      llm_model: 'glm-4-flash',
    }

    render(<LlmSetupStep />)

    expect(screen.getAllByRole('combobox')[0]).toHaveValue('company')
    await waitFor(() => {
      expect(mockStore.updateConfig).toHaveBeenCalledWith({
        llm_provider: 'company',
        llm_base_url: LLM_DEFAULT_CONFIG.company.baseUrl,
        llm_model: LLM_DEFAULT_CONFIG.company.model,
      })
    })
  })

  it('allows testing Ollama without an API key', async () => {
    render(<LlmSetupStep />)

    const button = screen.getByRole('button', { name: 'Test' })
    expect(screen.queryByPlaceholderText('API key')).not.toBeInTheDocument()
    expect(button).not.toBeDisabled()
    fireEvent.click(button)

    await waitFor(() =>
      expect(tauri.testLlmConnection).toHaveBeenCalledWith(
        '',
        'ollama',
        'http://localhost:11434/v1',
        'llama3.2',
      ),
    )
  })

  it('lets the company gateway fetch models without a key while keeping the key field optional', () => {
    mockStore.config = {
      llm_provider: 'company',
      llm_api_key: '',
      llm_base_url: 'https://llm.company.internal/v1',
      llm_model: 'default',
    }

    render(<LlmSetupStep />)

    // The key field is offered (a gateway may be deployed with a token)…
    expect(screen.getByPlaceholderText('API key')).toBeInTheDocument()
    // …but the model fetch button is never blocked on the key.
    expect(screen.getByTitle('Fetch models')).not.toBeDisabled()
  })
})
