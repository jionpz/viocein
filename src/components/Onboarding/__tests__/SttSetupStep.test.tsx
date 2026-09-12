import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { SttSetupStep } from '../SttSetupStep'

const mockStore = {
  config: {
    stt_provider: 'custom-whisper',
    stt_api_key: '',
    stt_custom_api_key: '',
    stt_custom_base_url: 'http://localhost:8000/v1',
    stt_custom_model: 'Systran/faster-whisper-large-v3',
  },
  updateConfig: vi.fn(),
  sttTestStatus: 'idle',
  setSttTestStatus: vi.fn(),
}

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) =>
      ({
        'onboarding.stt.serviceLabel': 'Service',
        'onboarding.stt.apiKeyLabel': 'API key',
        'onboarding.stt.apiKeyPlaceholder': 'API key',
        'onboarding.stt.testButton': 'Test',
        'onboarding.stt.connectionOk': 'OK',
        'onboarding.stt.connectionFail': 'Failed',
        'onboarding.stt.customWhisperConfigured': 'Custom Whisper',
        'providers.stt.customWhisper': 'Custom Whisper',
      })[key] ?? key,
  }),
}))

vi.mock('../../../stores/appStore', () => ({
  useAppStore: (selector: any) => selector(mockStore),
}))

vi.mock('../../../lib/tauri', () => ({
  testSttConnection: vi.fn().mockResolvedValue(true),
}))

beforeEach(() => {
  mockStore.config = {
    stt_provider: 'custom-whisper',
    stt_api_key: '',
    stt_custom_api_key: '',
    stt_custom_base_url: 'http://localhost:8000/v1',
    stt_custom_model: 'Systran/faster-whisper-large-v3',
  }
  mockStore.updateConfig = vi.fn()
  mockStore.sttTestStatus = 'idle'
  mockStore.setSttTestStatus = vi.fn()
})

afterEach(() => cleanup())

describe('SttSetupStep', () => {
  it('offers only local providers and falls back from a removed provider', () => {
    mockStore.config = { ...mockStore.config, stt_provider: 'removed-provider' }

    render(<SttSetupStep />)

    const providerSelect = screen.getByRole('combobox')
    expect(providerSelect.querySelectorAll('option')).toHaveLength(2)
    expect(providerSelect.querySelector('option[value="custom-whisper"]')).not.toBeNull()
    expect(providerSelect.querySelector('option[value="apple-speech"]')).not.toBeNull()
  })

  it('preserves an existing Custom Whisper setup instead of switching providers', () => {
    mockStore.config = {
      ...mockStore.config,
      stt_provider: 'custom-whisper',
      stt_custom_base_url: 'http://localhost:9000/v1',
      stt_custom_model: 'local-large-v3',
    }

    render(<SttSetupStep />)

    expect(screen.getByText('Custom Whisper')).toBeInTheDocument()
    expect(screen.getByText('http://localhost:9000/v1')).toBeInTheDocument()
    expect(screen.getByText('local-large-v3')).toBeInTheDocument()
    expect(mockStore.updateConfig).not.toHaveBeenCalled()
  })

  it('tests Custom Whisper with its configured endpoint and model', async () => {
    const tauri = await import('../../../lib/tauri')
    mockStore.config = {
      ...mockStore.config,
      stt_provider: 'custom-whisper',
      stt_custom_api_key: 'custom-secret',
      stt_custom_base_url: 'http://localhost:9000/v1',
      stt_custom_model: 'local-large-v3',
    }

    render(<SttSetupStep />)
    fireEvent.click(screen.getByRole('button', { name: 'Test' }))

    await waitFor(() => {
      expect(tauri.testSttConnection).toHaveBeenCalledWith(
        'custom-secret',
        'custom-whisper',
        'http://localhost:9000/v1',
        'local-large-v3',
      )
    })
  })
})
