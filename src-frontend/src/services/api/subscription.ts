import { loggedInvoke } from './core';
// ==================== Subscription (Freemium) ====================

export interface SubscriptionStatus {
  user_id: string;
  tier: string;
  status: string;
  expires_at?: string;
}

export const getSubscriptionStatus = () =>
  loggedInvoke<SubscriptionStatus>('get_subscription_status');

export const devUpgradeSubscription = (tier: string) =>
  loggedInvoke<SubscriptionStatus>('dev_upgrade_subscription', { tier });

export const devDowngradeSubscription = () =>
  loggedInvoke<SubscriptionStatus>('dev_downgrade_subscription');
