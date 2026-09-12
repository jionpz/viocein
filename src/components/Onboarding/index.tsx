import { AnimatePresence, motion } from 'framer-motion'
import { useTranslation } from 'react-i18next'
import { useAppStore } from '../../stores/appStore'
import { saveOnboardingCompleted, updateConfig as saveConfig } from '../../lib/tauri'
import { OnboardingLayout } from './OnboardingLayout'
import { WelcomeStep } from './WelcomeStep'
import { SttSetupStep } from './SttSetupStep'
import { LlmSetupStep } from './LlmSetupStep'
import { PermissionsStep } from './PermissionsStep'
import { DoneStep } from './DoneStep'
import { slideRight } from '../../lib/animations'

// Internal local-only onboarding: welcome, local STT, company LLM,
// permissions, done. No account, cloud mode, or checkout steps.
const TOTAL_STEPS = 5

export function Onboarding() {
  const { t } = useTranslation()
  const rawStep = useAppStore((s) => s.onboardingStep)
  const setStep = useAppStore((s) => s.setOnboardingStep)
  const setOnboardingCompleted = useAppStore((s) => s.setOnboardingCompleted)
  const sttTestStatus = useAppStore((s) => s.sttTestStatus)
  const llmTestStatus = useAppStore((s) => s.llmTestStatus)

  // Defensive clamp: a stale or out-of-range step must never crash the wizard.
  const step = Number.isInteger(rawStep) ? Math.min(Math.max(rawStep, 0), TOTAL_STEPS - 1) : 0

  const canNext = (() => {
    switch (step) {
      case 0:
        return true
      case 1:
        return sttTestStatus === 'success'
      case 2:
        return llmTestStatus === 'success'
      default:
        return true
    }
  })()

  const titles = [
    { title: t('onboarding.steps.welcome'), subtitle: t('onboarding.steps.welcomeSub') },
    {
      title: t('onboarding.steps.speechRecognition'),
      subtitle: t('onboarding.steps.speechRecognitionSub'),
    },
    { title: t('onboarding.steps.aiPolish'), subtitle: t('onboarding.steps.aiPolishSub') },
    { title: t('onboarding.steps.permissions'), subtitle: t('onboarding.steps.permissionsSub') },
    { title: t('onboarding.steps.setupComplete'), subtitle: undefined },
  ]

  const config = useAppStore((s) => s.config)

  const handleNext = async () => {
    if (step < TOTAL_STEPS - 1) {
      try {
        await saveConfig(config)
      } catch {
        // Best-effort save — continue navigation even if save fails
      }
      setStep(step + 1)
      return
    }

    await saveConfig(config)
    await saveOnboardingCompleted()
    setOnboardingCompleted(true)
  }

  const handleBack = async () => {
    if (step === 0) return
    try {
      await saveConfig(config)
    } catch {
      // Best-effort save
    }
    setStep(step - 1)
  }

  return (
    <OnboardingLayout
      step={step}
      totalSteps={TOTAL_STEPS}
      title={titles[step].title}
      subtitle={titles[step].subtitle}
      canNext={canNext}
      canBack={step > 0}
      nextLabel={
        step === TOTAL_STEPS - 1 ? t('onboarding.steps.getStarted') : t('onboarding.layout.next')
      }
      onNext={handleNext}
      onBack={handleBack}
    >
      <AnimatePresence mode="wait">
        <motion.div
          key={step}
          variants={slideRight}
          initial="initial"
          animate="animate"
          exit="exit"
          transition={{ duration: 0.2 }}
        >
          {step === 0 && <WelcomeStep />}
          {step === 1 && <SttSetupStep />}
          {step === 2 && <LlmSetupStep />}
          {step === 3 && <PermissionsStep />}
          {step === 4 && <DoneStep />}
        </motion.div>
      </AnimatePresence>
    </OnboardingLayout>
  )
}
