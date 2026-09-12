// App metadata
export const UI_LANGUAGES = [
  { value: 'en', label: 'English' },
  { value: 'zh', label: '中文' },
  { value: 'ja', label: '日本語' },
  { value: 'ko', label: '한국어' },
  { value: 'fr', label: 'Français' },
  { value: 'de', label: 'Deutsch' },
  { value: 'es', label: 'Español' },
  { value: 'pt', label: 'Português' },
  { value: 'ru', label: 'Русский' },
  { value: 'it', label: 'Italiano' },
] as const

export const APP_NAME = 'viocein'
export const APP_VERSION = import.meta.env.VITE_APP_VERSION ?? 'v0.1.42'

export const CUSTOM_WHISPER_PROVIDER = 'custom-whisper' as const
export const APPLE_SPEECH_PROVIDER = 'apple-speech' as const

export const CUSTOM_STT_DEFAULTS = {
  preset: 'speaches',
  baseUrl: 'http://localhost:8000/v1',
  model: 'Systran/faster-whisper-large-v3',
} as const

export const CUSTOM_STT_PRESETS = [
  {
    value: 'speaches',
    labelKey: 'settings.customSttPresetSpeaches',
    baseUrl: CUSTOM_STT_DEFAULTS.baseUrl,
    model: CUSTOM_STT_DEFAULTS.model,
  },
  {
    value: 'custom',
    labelKey: 'settings.customSttPresetCustom',
  },
] as const

export const STT_PROVIDERS: { value: string; labelKey: string }[] = [
  { value: CUSTOM_WHISPER_PROVIDER, labelKey: 'providers.stt.customWhisper' },
  { value: APPLE_SPEECH_PROVIDER, labelKey: 'providers.stt.appleSpeech' },
] as const

export const ONBOARDING_STT_PROVIDERS = STT_PROVIDERS

export const COMPANY_LLM_PROVIDER = 'company' as const

export const LLM_PROVIDERS: { value: string; labelKey: string }[] = [
  { value: COMPANY_LLM_PROVIDER, labelKey: 'providers.llm.company' },
  { value: 'ollama', labelKey: 'providers.llm.ollama' },
] as const

export const ONBOARDING_LLM_PROVIDERS = LLM_PROVIDERS

// Placeholder values only. The company gateway URL is entered by the operator
// in Settings/onboarding and is never baked into the binary.
export const LLM_DEFAULT_CONFIG: Record<string, { baseUrl: string; model: string }> = {
  company: { baseUrl: '', model: 'default' },
  ollama: { baseUrl: 'http://localhost:11434/v1', model: 'llama3.2' },
}

// Session-scoped memory of what each provider was last configured with, so
// switching the provider selector back and forth does not silently drop a
// gateway URL the operator just typed.
const LLM_LAST_CONNECTION: Record<string, { baseUrl: string; model: string }> = {}

export function rememberLlmConnection(provider: string, baseUrl: string, model: string): void {
  LLM_LAST_CONNECTION[provider] = { baseUrl: baseUrl.trim(), model: model.trim() }
}

export function recallLlmConnection(provider: string): { baseUrl: string; model: string } | undefined {
  return LLM_LAST_CONNECTION[provider]
}

// The company OpenAI-compatible gateway may be deployed with or without a
// bearer token, so the key field is shown for company but is never required.
// Ollama is loopback-only and never uses a key.
export function llmProviderRequiresApiKey(provider: string): boolean {
  return provider.trim().toLowerCase() === 'company'
}

export const LANGUAGES: { value: string; label?: string; labelKey?: string }[] = [
  { value: 'multi', labelKey: 'settings.autoDetect' },
  { value: 'zh', label: '中文' },
  { value: 'en', label: 'English' },
  { value: 'ja', label: '日本語' },
  { value: 'ko', label: '한국어' },
  { value: 'fr', label: 'Français' },
  { value: 'de', label: 'Deutsch' },
  { value: 'es', label: 'Español' },
  { value: 'pt', label: 'Português' },
  { value: 'ru', label: 'Русский' },
  { value: 'ar', label: 'العربية' },
  { value: 'hi', label: 'हिन्दी' },
  { value: 'th', label: 'ไทย' },
  { value: 'vi', label: 'Tiếng Việt' },
  { value: 'it', label: 'Italiano' },
  { value: 'nl', label: 'Nederlands' },
  { value: 'tr', label: 'Türkçe' },
  { value: 'pl', label: 'Polski' },
  { value: 'uk', label: 'Українська' },
  { value: 'id', label: 'Bahasa Indonesia' },
  { value: 'ms', label: 'Bahasa Melayu' },
]

export const TARGET_LANGUAGES: { value: string; label: string; labelKey?: string }[] = [
  { value: 'en', label: 'English' },
  { value: 'zh', label: '中文' },
  { value: 'ja', label: '日本語' },
  { value: 'ko', label: '한국어' },
  { value: 'fr', label: 'Français' },
  { value: 'de', label: 'Deutsch' },
  { value: 'es', label: 'Español' },
  { value: 'pt', label: 'Português' },
  { value: 'ru', label: 'Русский' },
  { value: 'ar', label: 'العربية' },
  { value: 'hi', label: 'हिन्दी' },
  { value: 'th', label: 'ไทย' },
  { value: 'vi', label: 'Tiếng Việt' },
  { value: 'it', label: 'Italiano' },
  { value: 'nl', label: 'Nederlands' },
  { value: 'tr', label: 'Türkçe' },
  { value: 'pl', label: 'Polski' },
  { value: 'uk', label: 'Українська' },
  { value: 'id', label: 'Bahasa Indonesia' },
  { value: 'ms', label: 'Bahasa Melayu' },
]
