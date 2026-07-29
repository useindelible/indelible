//! Managed-quota reservation/settlement. Counters use saturating arithmetic so a
//! silent overflow/underflow can't corrupt billing accounting;
//! `arithmetic_side_effects` is denied module-wide to enforce that.
#![deny(clippy::arithmetic_side_effects)]

use chrono::Utc;
use ind_domain::{BillingUsageEvent, TtsChunkRecordId, TtsProvider, TtsProviderUsage, UserId};
use uuid::Uuid;

use super::service::SynthesisService;
use super::timings::{current_month_bounds, estimate_audio_seconds};
use super::types::{
    ManagedQuotaReservation, TTS_MANAGED_CHARS_QUOTA, TTS_MANAGED_COST_UNITS_QUOTA,
    TTS_MANAGED_SECONDS_QUOTA,
};
use crate::AppError;

impl SynthesisService {
    pub(super) async fn reserve_managed_quota(
        &self,
        user_id: UserId,
        normalized_text: &str,
        character_limit_override: Option<i64>,
    ) -> Result<ManagedQuotaReservation, AppError> {
        let estimated_chars = normalized_text.chars().count() as i64;
        let (period_start, period_end) = current_month_bounds(Utc::now());
        let character_limit =
            character_limit_override.unwrap_or(self.managed_limits.monthly_characters);
        let mut reservation = ManagedQuotaReservation {
            period_start,
            period_end,
            character_limit,
            seconds_limit: self.managed_limits.monthly_seconds,
            cost_units_limit: self.managed_limits.monthly_cost_units,
            characters: 0,
            seconds: 0,
            cost_units: 0,
        };

        let estimated_seconds = estimate_audio_seconds(normalized_text);
        let estimated_cost_units = estimated_chars;

        self.reserve_axis(
            user_id,
            TTS_MANAGED_CHARS_QUOTA,
            reservation.period_start,
            reservation.period_end,
            reservation.character_limit,
            estimated_chars,
        )
        .await?;
        reservation.characters = estimated_chars;

        if let Err(err) = self
            .reserve_axis(
                user_id,
                TTS_MANAGED_SECONDS_QUOTA,
                reservation.period_start,
                reservation.period_end,
                reservation.seconds_limit,
                estimated_seconds,
            )
            .await
        {
            self.release_managed_quota(user_id, &reservation).await?;
            return Err(err);
        }
        reservation.seconds = estimated_seconds;

        if let Err(err) = self
            .reserve_axis(
                user_id,
                TTS_MANAGED_COST_UNITS_QUOTA,
                reservation.period_start,
                reservation.period_end,
                reservation.cost_units_limit,
                estimated_cost_units,
            )
            .await
        {
            self.release_managed_quota(user_id, &reservation).await?;
            return Err(err);
        }
        reservation.cost_units = estimated_cost_units;

        Ok(reservation)
    }

    pub(super) async fn reserve_axis(
        &self,
        user_id: UserId,
        quota_name: &'static str,
        period_start: chrono::DateTime<Utc>,
        period_end: chrono::DateTime<Utc>,
        limit_value: i64,
        amount: i64,
    ) -> Result<(), AppError> {
        if amount <= 0 {
            return Ok(());
        }
        let reserved = self
            .usage_counters
            .try_increment_window_by(
                user_id,
                quota_name,
                period_start,
                period_end,
                limit_value,
                amount,
            )
            .await?;
        if reserved.is_some() {
            Ok(())
        } else {
            Err(AppError::QuotaExceeded { quota: quota_name })
        }
    }

    pub(super) async fn ensure_actual_usage_reserved(
        &self,
        user_id: UserId,
        mut reservation: ManagedQuotaReservation,
        usage: &TtsProviderUsage,
    ) -> Result<ManagedQuotaReservation, AppError> {
        let actual_characters = usage.characters.unwrap_or(reservation.characters).max(0);
        let actual_seconds = usage
            .audio_seconds
            .map(|seconds| seconds.max(0.0).round() as i64)
            .unwrap_or(reservation.seconds);
        let actual_cost_units = usage.cost_units.unwrap_or(reservation.cost_units).max(0);

        if let Err(err) = self
            .reserve_usage_delta(
                user_id,
                TTS_MANAGED_CHARS_QUOTA,
                reservation.period_start,
                reservation.period_end,
                reservation.character_limit,
                &mut reservation.characters,
                actual_characters,
            )
            .await
        {
            self.release_managed_quota(user_id, &reservation).await?;
            return Err(err);
        };

        if let Err(err) = self
            .reserve_usage_delta(
                user_id,
                TTS_MANAGED_SECONDS_QUOTA,
                reservation.period_start,
                reservation.period_end,
                reservation.seconds_limit,
                &mut reservation.seconds,
                actual_seconds,
            )
            .await
        {
            self.release_managed_quota(user_id, &reservation).await?;
            return Err(err);
        };

        if let Err(err) = self
            .reserve_usage_delta(
                user_id,
                TTS_MANAGED_COST_UNITS_QUOTA,
                reservation.period_start,
                reservation.period_end,
                reservation.cost_units_limit,
                &mut reservation.cost_units,
                actual_cost_units,
            )
            .await
        {
            self.release_managed_quota(user_id, &reservation).await?;
            return Err(err);
        };

        Ok(reservation)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) async fn reserve_usage_delta(
        &self,
        user_id: UserId,
        quota_name: &'static str,
        period_start: chrono::DateTime<Utc>,
        period_end: chrono::DateTime<Utc>,
        limit_value: i64,
        reserved_amount: &mut i64,
        actual_amount: i64,
    ) -> Result<(), AppError> {
        let delta = actual_amount.saturating_sub(*reserved_amount);
        if delta > 0 {
            self.reserve_axis(
                user_id,
                quota_name,
                period_start,
                period_end,
                limit_value,
                delta,
            )
            .await?;
            *reserved_amount = reserved_amount.saturating_add(delta);
        }
        Ok(())
    }

    pub(super) async fn release_managed_quota(
        &self,
        user_id: UserId,
        reservation: &ManagedQuotaReservation,
    ) -> Result<(), AppError> {
        self.adjust_reserved_axis(
            user_id,
            TTS_MANAGED_CHARS_QUOTA,
            reservation,
            reservation.character_limit,
            reservation.characters.saturating_neg(),
        )
        .await?;
        self.adjust_reserved_axis(
            user_id,
            TTS_MANAGED_SECONDS_QUOTA,
            reservation,
            reservation.seconds_limit,
            reservation.seconds.saturating_neg(),
        )
        .await?;
        self.adjust_reserved_axis(
            user_id,
            TTS_MANAGED_COST_UNITS_QUOTA,
            reservation,
            reservation.cost_units_limit,
            reservation.cost_units.saturating_neg(),
        )
        .await
    }

    pub(super) async fn release_over_reserved_quota(
        &self,
        user_id: UserId,
        reservation: &mut ManagedQuotaReservation,
        usage: &TtsProviderUsage,
    ) -> Result<(), AppError> {
        let actual_characters = usage.characters.unwrap_or(reservation.characters).max(0);
        let actual_seconds = usage
            .audio_seconds
            .map(|seconds| seconds.max(0.0).round() as i64)
            .unwrap_or(reservation.seconds);
        let actual_cost_units = usage.cost_units.unwrap_or(reservation.cost_units).max(0);

        let char_delta = actual_characters.saturating_sub(reservation.characters);
        self.adjust_reserved_axis(
            user_id,
            TTS_MANAGED_CHARS_QUOTA,
            reservation,
            reservation.character_limit,
            char_delta,
        )
        .await?;
        reservation.characters = actual_characters;

        let seconds_delta = actual_seconds.saturating_sub(reservation.seconds);
        self.adjust_reserved_axis(
            user_id,
            TTS_MANAGED_SECONDS_QUOTA,
            reservation,
            reservation.seconds_limit,
            seconds_delta,
        )
        .await?;
        reservation.seconds = actual_seconds;

        let cost_delta = actual_cost_units.saturating_sub(reservation.cost_units);
        self.adjust_reserved_axis(
            user_id,
            TTS_MANAGED_COST_UNITS_QUOTA,
            reservation,
            reservation.cost_units_limit,
            cost_delta,
        )
        .await?;
        reservation.cost_units = actual_cost_units;

        Ok(())
    }

    pub(super) async fn adjust_reserved_axis(
        &self,
        user_id: UserId,
        quota_name: &'static str,
        reservation: &ManagedQuotaReservation,
        limit_value: i64,
        amount: i64,
    ) -> Result<(), AppError> {
        if amount == 0 {
            return Ok(());
        }
        self.usage_counters
            .increment_window_by(
                user_id,
                quota_name,
                reservation.period_start,
                reservation.period_end,
                limit_value,
                amount,
            )
            .await?;
        Ok(())
    }

    pub(super) async fn record_usage(
        &self,
        user_id: UserId,
        provider: TtsProvider,
        chunk_id: TtsChunkRecordId,
        usage: &TtsProviderUsage,
    ) -> Result<Option<BillingUsageEvent>, AppError> {
        let characters = usage.characters.unwrap_or(0).max(0);
        let audio_seconds = usage.audio_seconds.unwrap_or(0.0).max(0.0);
        let cost_units = usage.cost_units.unwrap_or(0).max(0);
        let now = Utc::now();
        let event = BillingUsageEvent {
            id: Uuid::now_v7(),
            user_id,
            billing_account_id: None,
            product_area: "tts".into(),
            event_type: "tts_synthesis".into(),
            provider: Some(provider.as_str().into()),
            billing_mode: "managed".into(),
            resource_type: "tts_chunk".into(),
            resource_id: Some(chunk_id.into_uuid()),
            units: serde_json::json!({
                "characters": characters,
                "audio_seconds": audio_seconds,
                "cost_units": cost_units,
            }),
            cost_units,
            amount_cents: None,
            currency: None,
            idempotency_key: format!("tts_synthesis:{chunk_id}"),
            metadata: serde_json::json!({}),
            occurred_at: now,
            created_at: now,
        };
        self.billing_usage_events.insert(&event).await.map(Some)
    }
}
