import React from 'react'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { Onboarding } from '../index'

const mockStore = {
  onboardingStep: 0,
  setOnboardingStep: vi.fn(),
  setOnboardingCompleted: vi.fn(),
  sttTestStatus: 'idle',
  llmTestStatus: 'idle',
  config: {},
}

vi.mock('framer-motion', () => ({
  AnimatePresence: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  motion: {
    div: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  },
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))

vi.mock('../OnboardingLayout', () => ({
  OnboardingLayout: ({
    children,
    onBack,
    onNext,
  }: {
    children: React.ReactNode
    onBack: () => void
    onNext: () => void
  }) => (
    <div>
      <button type="button" onClick={onBack}>
        Back
      </button>
      <button type="button" onClick={onNext}>
        Next
      </button>
      {children}
    </div>
  ),
}))

vi.mock('../WelcomeStep', () => ({ WelcomeStep: () => <div>Welcome</div> }))
vi.mock('../SttSetupStep', () => ({ SttSetupStep: () => <div>STT</div> }))
vi.mock('../LlmSetupStep', () => ({ LlmSetupStep: () => <div>LLM</div> }))
vi.mock('../PermissionsStep', () => ({ PermissionsStep: () => <div>Permissions</div> }))
vi.mock('../DoneStep', () => ({ DoneStep: () => <div>Done</div> }))

vi.mock('../../../stores/appStore', () => ({
  useAppStore: (selector: (state: typeof mockStore) => unknown) => selector(mockStore),
}))

vi.mock('../../../lib/tauri', () => ({
  updateConfig: vi.fn().mockResolvedValue(undefined),
  saveOnboardingCompleted: vi.fn().mockResolvedValue(undefined),
}))

beforeEach(() => {
  mockStore.onboardingStep = 0
  mockStore.sttTestStatus = 'idle'
  mockStore.llmTestStatus = 'idle'
  mockStore.setOnboardingStep.mockReset()
  mockStore.setOnboardingCompleted.mockReset()
})

afterEach(() => cleanup())

describe('Onboarding local-only navigation', () => {
  it('starts on the welcome step', () => {
    render(<Onboarding />)

    expect(screen.getByText('Welcome')).toBeInTheDocument()
  })

  it('returns from the LLM step to the local STT step', async () => {
    mockStore.onboardingStep = 2
    render(<Onboarding />)

    fireEvent.click(screen.getByRole('button', { name: 'Back' }))

    await waitFor(() => expect(mockStore.setOnboardingStep).toHaveBeenCalledWith(1))
  })

  it('returns from the done step to permissions', async () => {
    mockStore.onboardingStep = 4
    render(<Onboarding />)

    fireEvent.click(screen.getByRole('button', { name: 'Back' }))

    await waitFor(() => expect(mockStore.setOnboardingStep).toHaveBeenCalledWith(3))
  })

  it('advances from welcome to the local STT step without a cloud mode step', async () => {
    render(<Onboarding />)

    fireEvent.click(screen.getByRole('button', { name: 'Next' }))

    await waitFor(() => expect(mockStore.setOnboardingStep).toHaveBeenCalledWith(1))
  })
})
