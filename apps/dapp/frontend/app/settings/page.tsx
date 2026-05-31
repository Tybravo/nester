"use client";

import { useState } from "react";
import { AppShell } from "@/components/app-shell";
import { OnboardingWizard } from "@/components/onboarding/OnboardingWizard";
import { useWallet } from "@/components/wallet-provider";

export default function SettingsPage() {
  const { isConnected } = useWallet();
  const [wizardOpen, setWizardOpen] = useState(false);

  if (!isConnected) {
    return (
      <AppShell>
        <p className="text-sm text-muted-foreground">Connect your wallet to manage settings.</p>
      </AppShell>
    );
  }

  return (
    <AppShell>
      <h1 className="font-heading text-3xl font-light">Settings</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        Manage your profile and onboarding preferences.
      </p>

      <section className="mt-8 rounded-2xl border border-border bg-white p-6">
        <h2 className="text-lg font-medium">Onboarding</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          Re-run the guided setup to update your savings goal and risk profile.
        </p>
        <button
          type="button"
          onClick={() => setWizardOpen(true)}
          className="mt-4 rounded-full bg-foreground px-5 py-2.5 text-sm font-medium text-background"
        >
          Restart onboarding wizard
        </button>
      </section>

      <OnboardingWizard
        open={wizardOpen}
        onClose={() => setWizardOpen(false)}
        onComplete={() => setWizardOpen(false)}
      />
    </AppShell>
  );
}
