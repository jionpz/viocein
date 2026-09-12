import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, waitFor, cleanup, within } from '@testing-library/react'
import { SttPane } from '../SttPane'
import * as tauri from '../../../lib/tauri'

vi.mock('../../../lib/tauri')

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, string | number>) => {
      const translations: Record<string, string> = {
        'settings.provider': 'Provider',
        'settings.apiKey': 'API Key',
        'settings.test': 'Test',
        'settings.enterApiKey': 'Enter API Key',
        'settings.connectionSuccess': 'Connection successful',
        'settings.connectionFailed': 'Connection failed',
        'settings.storedLocally': 'Stored locally',
        'settings.credentialSaveFailed': `Credential save failed: ${values?.details ?? ''}`,
        'settings.sttLanguage': 'STT Language',
        'settings.maxRecordingDuration': 'Single recording duration',
        'settings.healthChecking': 'Checking…',
        'settings.customSttPreset': 'Preset',
        'settings.customSttPresetSpeaches': 'Speaches',
        'settings.customSttPresetCustom': 'Custom OpenAI-compatible',
        'settings.customSttBaseUrl': 'Base URL',
        'settings.customSttBaseUrlPlaceholder': 'http://localhost:8000/v1',
        'settings.customSttModel': 'Model',
        'settings.customSttModelPlaceholder': 'Systran/faster-whisper-large-v3',
        'settings.customSttApiKeyOptional': 'API Key (optional)',
        'settings.customSttSetupHint':
          'Start your local OpenAI-compatible STT server first, then test the connection here.',
        'settings.localSttReady': 'Local endpoint ready',
        'settings.localSttNeedsSetup': 'Local endpoint needs setup',
        'settings.appleSpeechReady': 'Apple Speech ready',
        'settings.appleSpeechUnavailable': 'Apple Speech unavailable',
        'settings.autoDetect': 'Auto detect',
        'recordingLimits.auto': 'Auto',
        'recordingLimits.custom': 'Custom',
        'recordingLimits.customDuration': 'Custom duration',
        'recordingLimits.allowedRange': 'Supported range: {{min}}–{{max}}.',
        'recordingLimits.allowedRangeWithReason': 'Range: {{min}}–{{max}}. {{reason}}',
        'recordingLimits.corrected': 'This provider will use {{duration}}.',
        'recordingLimits.currentSelectionWithLimit':
          'Selected: {{current}}; app limit: {{max}}. {{reason}}',
        'recordingLimits.providerFixedLimit': 'This provider allows recordings up to {{max}}.',
        'recordingLimits.durationSeconds': '{{count}} seconds',
        'recordingLimits.durationMinute': '{{count}} minute',
        'recordingLimits.durationMinutes': '{{count}} minutes',
        'recordingLimits.secondsUnit': 'seconds',
        'recordingLimits.presets': 'Recording limit presets',
        'recordingLimits.numericEntry': 'Custom duration…',
        'recordingLimits.loading': 'Loading recording limits…',
        'recordingLimits.reasons.appleSpeech': 'Apple Speech session limit',
        'recordingLimits.reasons.unknownUpstream': 'Upstream limit unknown',
        'recordingLimits.reasons.unknownProvider': 'Provider limit unknown',
        'providers.stt.customWhisper': 'Local / Custom Whisper',
        'providers.stt.appleSpeech': 'Apple Speech (Local)',
      }
      return Object.entries(values ?? {}).reduce(
        (text, [name, value]) => text.replace(`{{${name}}}`, String(value)),
        translations[key] || key,
      )
    },
  }),
}))

const mockAppStore = {
  config: {
    stt_provider: 'custom-whisper' as string,
    stt_api_key: '',
    stt_custom_api_key: '',
    stt_language: 'en',
    stt_custom_preset: 'speaches',
    stt_custom_base_url: 'http://localhost:8000/v1',
    stt_custom_model: 'Systran/faster-whisper-large-v3',
    recording_limit_mode: 'auto' as 'auto' | 'custom',
    custom_recording_limit_seconds: 120,
    max_recording_seconds: 120,
  },
  updateConfig: vi.fn(),
  sttTestStatus: 'idle' as 'idle' | 'testing' | 'success' | 'error',
  setSttTestStatus: vi.fn(),
  sttLatencyMs: null as number | null,
  setSttLatencyMs: vi.fn(),
  platformCapabilities: {
    os: 'macos',
    sessionType: 'unknown',
    globalHotkeyReliable: true,
    keyboardOutputReliable: true,
    clipboardAutoPasteReliable: true,
  },
}

vi.mock('../../../stores/appStore', () => ({
  isMacPlatform: () => true,
  useAppStore: (selector: (state: typeof mockAppStore) => unknown) =>
    typeof selector === 'function' ? selector(mockAppStore) : mockAppStore,
}))

const whisperCapability = {
  capability: {
    registryVersion: 1,
    providerId: 'custom-whisper',
    transport: 'fileUpload' as const,
    recommendedMaxSeconds: 120,
    hardMaxSeconds: 720,
    maxUploadBytes: 24 * 1024 * 1024,
    source: 'unknownUpstream' as const,
    explanationKey: 'recordingLimits.reasons.unknownUpstream',
  },
  mode: 'auto' as const,
  requestedSeconds: 120,
  effectiveMaxSeconds: 120,
}

function resetStore() {
  mockAppStore.config = {
    stt_provider: 'custom-whisper',
    stt_api_key: '',
    stt_custom_api_key: '',
    stt_language: 'en',
    stt_custom_preset: 'speaches',
    stt_custom_base_url: 'http://localhost:8000/v1',
    stt_custom_model: 'Systran/faster-whisper-large-v3',
    recording_limit_mode: 'auto',
    custom_recording_limit_seconds: 120,
    max_recording_seconds: 120,
  }
  mockAppStore.sttTestStatus = 'idle'
  mockAppStore.sttLatencyMs = null
  mockAppStore.platformCapabilities = {
    os: 'macos',
    sessionType: 'unknown',
    globalHotkeyReliable: true,
    keyboardOutputReliable: true,
    clipboardAutoPasteReliable: true,
  }
}

describe('SttPane (internal local-only build)', () => {
  beforeEach(() => {
    resetStore()
    vi.clearAllMocks()
    vi.mocked(tauri.readCredential).mockResolvedValue(null)
    vi.mocked(tauri.setCredential).mockResolvedValue(undefined)
    vi.mocked(tauri.benchSttConnection).mockResolvedValue(12)
    vi.mocked(tauri.getSttRecordingCapability).mockResolvedValue(whisperCapability)
    vi.mocked(tauri.getSttProviderDiagnostics).mockResolvedValue({
      provider: 'custom-whisper',
      kind: 'localCompatible',
      endpoint: 'http://localhost:8000/v1/audio/transcriptions',
      model: 'Systran/faster-whisper-large-v3',
      requiresApiKey: false,
      apiKeyConfigured: false,
      ready: true,
      issues: [],
    })
  })

  afterEach(() => {
    cleanup()
    vi.clearAllMocks()
  })

  it('offers only local STT providers', () => {
    render(<SttPane />)
    const providerSelect = screen.getAllByRole('combobox')[0]
    expect(
      within(providerSelect).getByRole('option', { name: 'Local / Custom Whisper' }),
    ).toHaveValue('custom-whisper')
    expect(
      within(providerSelect).getByRole('option', { name: 'Apple Speech (Local)' }),
    ).toHaveValue('apple-speech')
    expect(within(providerSelect).getAllByRole('option')).toHaveLength(2)
  })

  it('shows the loopback endpoint and readiness for custom-whisper', async () => {
    render(<SttPane />)
    expect(await screen.findByText('Local endpoint ready')).toBeInTheDocument()
    expect(
      screen.getByText('http://localhost:8000/v1/audio/transcriptions'),
    ).toBeInTheDocument()
  })

  it('allows a keyless local Whisper server', () => {
    render(<SttPane />)
    expect(screen.getByRole('button', { name: /Test/i })).toBeEnabled()
  })

  it('benches the local STT endpoint when tested', async () => {
    render(<SttPane />)
    fireEvent.click(screen.getByRole('button', { name: /Test/i }))
    await waitFor(() => {
      expect(tauri.benchSttConnection).toHaveBeenCalledWith(
        '',
        'custom-whisper',
        'http://localhost:8000/v1',
        'Systran/faster-whisper-large-v3',
      )
    })
  })

  it('disables the test button when the endpoint or model is missing', () => {
    mockAppStore.config = { ...mockAppStore.config, stt_custom_model: '' }
    render(<SttPane />)
    expect(screen.getByRole('button', { name: /Test/i })).toBeDisabled()
  })

  it('updates config when the provider changes', () => {
    render(<SttPane />)
    fireEvent.change(screen.getAllByRole('combobox')[0], { target: { value: 'apple-speech' } })
    expect(mockAppStore.updateConfig).toHaveBeenCalledWith({ stt_provider: 'apple-speech' })
  })

  it('renders Apple Speech as a platform-gated local provider', async () => {
    mockAppStore.config = { ...mockAppStore.config, stt_provider: 'apple-speech' }
    vi.mocked(tauri.getSttProviderDiagnostics).mockResolvedValue({
      provider: 'apple-speech',
      kind: 'builtinLocal',
      endpoint: null,
      model: 'Apple Speech (en-US)',
      requiresApiKey: false,
      apiKeyConfigured: false,
      ready: true,
      issues: [],
    })
    render(<SttPane />)
    expect(await screen.findByText('Apple Speech ready')).toBeInTheDocument()
    expect(screen.queryByPlaceholderText('Enter API Key')).not.toBeInTheDocument()
  })

  it('requests the recording capability for the selected provider', async () => {
    render(<SttPane />)
    await waitFor(() => {
      expect(tauri.getSttRecordingCapability).toHaveBeenCalledWith('custom-whisper', 'auto', 120)
    })
  })

  it('updates the recording limit mode from the preset select', async () => {
    render(<SttPane />)
    const durationSelect = await screen.findByDisplayValue('Auto')
    fireEvent.change(durationSelect, { target: { value: '60' } })
    expect(mockAppStore.updateConfig).toHaveBeenCalledWith({
      recording_limit_mode: 'custom',
      custom_recording_limit_seconds: 60,
    })
  })
})
