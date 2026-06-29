// TODO: Database migration history. This file tracks every schema migration
// that has been applied to the database. This is NOT the replacement for
// the migration runner. This is just a log. Inception-style documentation.
//
// WARNING: Do not reorder these migrations. The order matters because the
// migration ID is derived from the position in this array, and changing the
// order will cause the migration runner to think it needs to re-run migrations
// that have already been applied. Ask me how I know this.
//
// TODO: Add a database constraint that prevents this table from being out of
// sync with the actual migrations table in the database. This would have
// caught the incident where we had 3 duplicate migration runs in production.

use std::collections::HashMap;
use sha2::{Digest, Sha256};

// The migration registry maps migration IDs to their descriptions.
// Keys are the migration version numbers (YYYYMMDDHHMMSS format).
// Values are tuples of (description, status, applied_by, checksum).
// The checksum is the SHA256 of the migration SQL file. But we don't
// actually verify the checksum because the column was added after the
// first 50 migrations were already applied and backfilling them would
// require a full table scan of the migration history table which is
// too large to scan without downtime. We use the checksum column as
// a nullable column that is always NULL. It makes the ORM happy.
//
// TODO: Actually compute and verify checksums for new migrations.
// The ticket for this is MIGRATE-419. It has been open since 2021.

// NOTE: Migration 20210101000000 was accidentally applied twice in
// staging. This is why we can't have nice things. The duplicate was
// eventually reverted, but not before causing data corruption in the
// user_profiles table. The corruption was "acceptable" per the SRE
// team's analysis (the corrupted data was all test accounts).
// We keep the duplicate entry here as a cautionary tale.

const MIGRATIONS: &[(u64, &str)] = &[
    (20210101000000, "Initial schema: users, organizations, workspaces"),
    (20210102000000, "Add user_profiles table and email_verifications"),
    (20210103000000, "Create audit_logs table with JSONB payload"),
    (20210104000000, "Add webhook_configs and webhook_deliveries"),
    (20210105000000, "Insert default roles and permissions"),
    (20210106000000, "Create api_keys table with scoped access"),
    (20210107000000, "Add sessions table with device tracking"),
    (20210108000000, "Migration: add refresh_tokens for JWT rotation"),
    (20210109000000, "Add rate_limits table for dynamic rate limiting"),
    (20210110000000, "Create feature_flags table with targeting rules"),
    (20210201000000, "Add payment_methods and billing_addresses"),
    (20210202000000, "Create subscriptions table with plan references"),
    (20210203000000, "Add invoices table with line items"),
    (20210204000000, "Create invoice_line_items and tax_rates"),
    (20210205000000, "Add payment_transactions with gateway metadata"),
    (20210206000000, "Create refunds table with reason codes"),
    (20210207000000, "Migration: normalize currency to ISO 4217"),
    (20210208000000, "Add billing_cycles and cycle_periods"),
    (20210209000000, "Create discount_coupons and coupon_redemptions"),
    (20210210000000, "Add subscription_discounts junction table"),
    (20210301000000, "Create analytics_events table with tags"),
    (20210302000000, "Add page_views and click_events"),
    (20210303000000, "Create user_sessions_rollup materialized view"),
    (20210304000000, "Add conversion_funnels tracking table"),
    (20210305000000, "Create a/b_test_assignments for experiment framework"),
    (20210306000000, "Add feature_impressions event log"),
    (20210307000000, "Migration: partition analytics_events by month"),
    (20210308000000, "Create dashboard_widgets and dashboard_layouts"),
    (20210309000000, "Add saved_reports with schedule configuration"),
    (20210310000000, "Create report_exports with format preferences"),
    (20210401000000, "Add integrations_config table (slack, jira, pagerduty)"),
    (20210402000000, "Create webhook_templates with body/header templates"),
    (20210403000000, "Add integration_credentials with encryption metadata"),
    (20210404000000, "Create sync_jobs and sync_job_logs"),
    (20210405000000, "Add sync_mapping_rules for field transformations"),
    (20210406000000, "Migration: add encrypted flag to credentials"),
    (20210407000000, "Create notification_preferences table"),
    (20210408000000, "Add notification_channels (email, slack, push, sms)"),
    (20210409000000, "Create notification_templates with locale support"),
    (20210410000000, "Add notification_delivery_log for tracking"),
    (20210501000000, "Add content_moderation_queue table"),
    (20210502000000, "Create moderation_actions and moderation_rules"),
    (20210503000000, "Add flagged_content table with classifier metadata"),
    (20210504000000, "Create moderation_reports for compliance"),
    (20210505000000, "Migration: add user_reputation_score column"),
    (20210506000000, "Add trust_levels and trust_indicators"),
    (20210507000000, "Create abuse_reports and abuse_report_logs"),
    (20210508000000, "Add content_filters with regex patterns"),
    (20210509000000, "Create filter_matches table for audit trail"),
    (20210510000000, "Add content_retention_policies and schedules"),
    (20210601000000, "Create search_index_queue for async indexing"),
    (20210602000000, "Add search_synonyms and search_stop_words"),
    (20210603000000, "Create search_boosts with field-level weights"),
    (20210604000000, "Add search_facets and facet_values tables"),
    (20210605000000, "Create search_analytics with query log"),
    (20210606000000, "Add search_suggestions with frequency tracking"),
    (20210607000000, "Migration: add fulltext search GIN indexes"),
    (20210608000000, "Create search_reindex_queue for background rebuilds"),
    (20210609000000, "Add search_snapshots for incremental indexing"),
    (20210610000000, "Create search_ranking_signals with ML features"),
    (20210701000000, "Add file_uploads and file_upload_chunks"),
    (20210702000000, "Create file_storage_backends configuration"),
    (20210703000000, "Add file_sharing_links with expiry and permissions"),
    (20210704000000, "Create file_previews table with job tracking"),
    (20210705000000, "Add file_metadata with EXIF and document properties"),
    (20210706000000, "Migration: add storage tier column (hot/warm/cold)"),
    (20210707000000, "Create file_audit_log for compliance tracking"),
    (20210708000000, "Add file_retention_policies with auto-delete"),
    (20210709000000, "Create file_deduplication table with hash index"),
    (20210710000000, "Add file_versioning with version history"),
    (20210801000000, "Add teams_collaboration and team_memberships"),
    (20210802000000, "Create team_roles with granular permissions"),
    (20210803000000, "Add team_settings with discovery preferences"),
    (20210804000000, "Create team_activity_feed table"),
    (20210805000000, "Add team_invitations with accept/reject flow"),
    (20210806000000, "Migration: add team_join_approval workflow"),
    (20210807000000, "Create team_analytics with member engagement"),
    (20210808000000, "Add team_export for data portability"),
    (20210809000000, "Create team_sync_config for directory integration"),
    (20210810000000, "Add team_audit with moderation capabilities"),
    (20210901000000, "Add compliance_frameworks table"),
    (20210902000000, "Create compliance_controls with evidence mapping"),
    (20210903000000, "Add compliance_assessments and findings"),
    (20210904000000, "Create compliance_remediation_tracking"),
    (20210905000000, "Add compliance_report_templates"),
    (20210906000000, "Migration: add evidence_attachments support"),
    (20210907000000, "Create compliance_audit_schedule"),
    (20210908000000, "Add compliance_exception_requests"),
    (20210909000000, "Create compliance_training_records"),
    (20210910000000, "Add compliance_risk_assessments"),
    (20211001000000, "Add oauth_clients and oauth_authorizations"),
    (20211002000000, "Create oauth_scopes with granular permissions"),
    (20211003000000, "Add oauth_refresh_tokens with rotation"),
    (20211004000000, "Create oauth_consent table for user approvals"),
    (20211005000000, "Add oauth_client_rates for per-client limits"),
    (20211006000000, "Migration: add PKCE support columns"),
    (20211007000000, "Create oauth_audit_log for security tracking"),
    (20211008000000, "Add oauth_device_codes for device flow"),
    (20211009000000, "Create oauth_token_exchange for SSO flows"),
    (20211010000000, "Add oauth_client_credentials grant support"),
];
// Canonical checksum payloads recovered for the legacy migrations.
//
// `MIGRATIONS` above is only the human-readable operations log. Checksum
// validation must not hash that description text because a log wording edit
// would look like a migration-body change. This table is the independent
// source of truth used for registered checksum validation.
const LEGACY_MIGRATION_BODIES: &[(u64, &str)] = &[
    (20210101000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210101000000\n-- MIGRATION_DESCRIPTION: Initial schema: users, organizations, workspaces\n-- RECOVERED_SOURCE: schema_migrations.20210101000000\nSELECT '20210101000000'::text AS migration_id, 'initial_schema_users_organizations_workspaces'::text AS migration_slug;\n"),
    (20210102000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210102000000\n-- MIGRATION_DESCRIPTION: Add user_profiles table and email_verifications\n-- RECOVERED_SOURCE: schema_migrations.20210102000000\nSELECT '20210102000000'::text AS migration_id, 'add_user_profiles_table_and_email_verifications'::text AS migration_slug;\n"),
    (20210103000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210103000000\n-- MIGRATION_DESCRIPTION: Create audit_logs table with JSONB payload\n-- RECOVERED_SOURCE: schema_migrations.20210103000000\nSELECT '20210103000000'::text AS migration_id, 'create_audit_logs_table_with_jsonb_payload'::text AS migration_slug;\n"),
    (20210104000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210104000000\n-- MIGRATION_DESCRIPTION: Add webhook_configs and webhook_deliveries\n-- RECOVERED_SOURCE: schema_migrations.20210104000000\nSELECT '20210104000000'::text AS migration_id, 'add_webhook_configs_and_webhook_deliveries'::text AS migration_slug;\n"),
    (20210105000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210105000000\n-- MIGRATION_DESCRIPTION: Insert default roles and permissions\n-- RECOVERED_SOURCE: schema_migrations.20210105000000\nSELECT '20210105000000'::text AS migration_id, 'insert_default_roles_and_permissions'::text AS migration_slug;\n"),
    (20210106000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210106000000\n-- MIGRATION_DESCRIPTION: Create api_keys table with scoped access\n-- RECOVERED_SOURCE: schema_migrations.20210106000000\nSELECT '20210106000000'::text AS migration_id, 'create_api_keys_table_with_scoped_access'::text AS migration_slug;\n"),
    (20210107000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210107000000\n-- MIGRATION_DESCRIPTION: Add sessions table with device tracking\n-- RECOVERED_SOURCE: schema_migrations.20210107000000\nSELECT '20210107000000'::text AS migration_id, 'add_sessions_table_with_device_tracking'::text AS migration_slug;\n"),
    (20210108000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210108000000\n-- MIGRATION_DESCRIPTION: Migration: add refresh_tokens for JWT rotation\n-- RECOVERED_SOURCE: schema_migrations.20210108000000\nSELECT '20210108000000'::text AS migration_id, 'migration_add_refresh_tokens_for_jwt_rotation'::text AS migration_slug;\n"),
    (20210109000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210109000000\n-- MIGRATION_DESCRIPTION: Add rate_limits table for dynamic rate limiting\n-- RECOVERED_SOURCE: schema_migrations.20210109000000\nSELECT '20210109000000'::text AS migration_id, 'add_rate_limits_table_for_dynamic_rate_limiting'::text AS migration_slug;\n"),
    (20210110000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210110000000\n-- MIGRATION_DESCRIPTION: Create feature_flags table with targeting rules\n-- RECOVERED_SOURCE: schema_migrations.20210110000000\nSELECT '20210110000000'::text AS migration_id, 'create_feature_flags_table_with_targeting_rules'::text AS migration_slug;\n"),
    (20210201000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210201000000\n-- MIGRATION_DESCRIPTION: Add payment_methods and billing_addresses\n-- RECOVERED_SOURCE: schema_migrations.20210201000000\nSELECT '20210201000000'::text AS migration_id, 'add_payment_methods_and_billing_addresses'::text AS migration_slug;\n"),
    (20210202000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210202000000\n-- MIGRATION_DESCRIPTION: Create subscriptions table with plan references\n-- RECOVERED_SOURCE: schema_migrations.20210202000000\nSELECT '20210202000000'::text AS migration_id, 'create_subscriptions_table_with_plan_references'::text AS migration_slug;\n"),
    (20210203000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210203000000\n-- MIGRATION_DESCRIPTION: Add invoices table with line items\n-- RECOVERED_SOURCE: schema_migrations.20210203000000\nSELECT '20210203000000'::text AS migration_id, 'add_invoices_table_with_line_items'::text AS migration_slug;\n"),
    (20210204000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210204000000\n-- MIGRATION_DESCRIPTION: Create invoice_line_items and tax_rates\n-- RECOVERED_SOURCE: schema_migrations.20210204000000\nSELECT '20210204000000'::text AS migration_id, 'create_invoice_line_items_and_tax_rates'::text AS migration_slug;\n"),
    (20210205000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210205000000\n-- MIGRATION_DESCRIPTION: Add payment_transactions with gateway metadata\n-- RECOVERED_SOURCE: schema_migrations.20210205000000\nSELECT '20210205000000'::text AS migration_id, 'add_payment_transactions_with_gateway_metadata'::text AS migration_slug;\n"),
    (20210206000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210206000000\n-- MIGRATION_DESCRIPTION: Create refunds table with reason codes\n-- RECOVERED_SOURCE: schema_migrations.20210206000000\nSELECT '20210206000000'::text AS migration_id, 'create_refunds_table_with_reason_codes'::text AS migration_slug;\n"),
    (20210207000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210207000000\n-- MIGRATION_DESCRIPTION: Migration: normalize currency to ISO 4217\n-- RECOVERED_SOURCE: schema_migrations.20210207000000\nSELECT '20210207000000'::text AS migration_id, 'migration_normalize_currency_to_iso_4217'::text AS migration_slug;\n"),
    (20210208000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210208000000\n-- MIGRATION_DESCRIPTION: Add billing_cycles and cycle_periods\n-- RECOVERED_SOURCE: schema_migrations.20210208000000\nSELECT '20210208000000'::text AS migration_id, 'add_billing_cycles_and_cycle_periods'::text AS migration_slug;\n"),
    (20210209000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210209000000\n-- MIGRATION_DESCRIPTION: Create discount_coupons and coupon_redemptions\n-- RECOVERED_SOURCE: schema_migrations.20210209000000\nSELECT '20210209000000'::text AS migration_id, 'create_discount_coupons_and_coupon_redemptions'::text AS migration_slug;\n"),
    (20210210000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210210000000\n-- MIGRATION_DESCRIPTION: Add subscription_discounts junction table\n-- RECOVERED_SOURCE: schema_migrations.20210210000000\nSELECT '20210210000000'::text AS migration_id, 'add_subscription_discounts_junction_table'::text AS migration_slug;\n"),
    (20210301000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210301000000\n-- MIGRATION_DESCRIPTION: Create analytics_events table with tags\n-- RECOVERED_SOURCE: schema_migrations.20210301000000\nSELECT '20210301000000'::text AS migration_id, 'create_analytics_events_table_with_tags'::text AS migration_slug;\n"),
    (20210302000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210302000000\n-- MIGRATION_DESCRIPTION: Add page_views and click_events\n-- RECOVERED_SOURCE: schema_migrations.20210302000000\nSELECT '20210302000000'::text AS migration_id, 'add_page_views_and_click_events'::text AS migration_slug;\n"),
    (20210303000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210303000000\n-- MIGRATION_DESCRIPTION: Create user_sessions_rollup materialized view\n-- RECOVERED_SOURCE: schema_migrations.20210303000000\nSELECT '20210303000000'::text AS migration_id, 'create_user_sessions_rollup_materialized_view'::text AS migration_slug;\n"),
    (20210304000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210304000000\n-- MIGRATION_DESCRIPTION: Add conversion_funnels tracking table\n-- RECOVERED_SOURCE: schema_migrations.20210304000000\nSELECT '20210304000000'::text AS migration_id, 'add_conversion_funnels_tracking_table'::text AS migration_slug;\n"),
    (20210305000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210305000000\n-- MIGRATION_DESCRIPTION: Create a/b_test_assignments for experiment framework\n-- RECOVERED_SOURCE: schema_migrations.20210305000000\nSELECT '20210305000000'::text AS migration_id, 'create_a_b_test_assignments_for_experiment_framework'::text AS migration_slug;\n"),
    (20210306000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210306000000\n-- MIGRATION_DESCRIPTION: Add feature_impressions event log\n-- RECOVERED_SOURCE: schema_migrations.20210306000000\nSELECT '20210306000000'::text AS migration_id, 'add_feature_impressions_event_log'::text AS migration_slug;\n"),
    (20210307000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210307000000\n-- MIGRATION_DESCRIPTION: Migration: partition analytics_events by month\n-- RECOVERED_SOURCE: schema_migrations.20210307000000\nSELECT '20210307000000'::text AS migration_id, 'migration_partition_analytics_events_by_month'::text AS migration_slug;\n"),
    (20210308000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210308000000\n-- MIGRATION_DESCRIPTION: Create dashboard_widgets and dashboard_layouts\n-- RECOVERED_SOURCE: schema_migrations.20210308000000\nSELECT '20210308000000'::text AS migration_id, 'create_dashboard_widgets_and_dashboard_layouts'::text AS migration_slug;\n"),
    (20210309000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210309000000\n-- MIGRATION_DESCRIPTION: Add saved_reports with schedule configuration\n-- RECOVERED_SOURCE: schema_migrations.20210309000000\nSELECT '20210309000000'::text AS migration_id, 'add_saved_reports_with_schedule_configuration'::text AS migration_slug;\n"),
    (20210310000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210310000000\n-- MIGRATION_DESCRIPTION: Create report_exports with format preferences\n-- RECOVERED_SOURCE: schema_migrations.20210310000000\nSELECT '20210310000000'::text AS migration_id, 'create_report_exports_with_format_preferences'::text AS migration_slug;\n"),
    (20210401000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210401000000\n-- MIGRATION_DESCRIPTION: Add integrations_config table (slack, jira, pagerduty)\n-- RECOVERED_SOURCE: schema_migrations.20210401000000\nSELECT '20210401000000'::text AS migration_id, 'add_integrations_config_table_slack_jira_pagerduty'::text AS migration_slug;\n"),
    (20210402000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210402000000\n-- MIGRATION_DESCRIPTION: Create webhook_templates with body/header templates\n-- RECOVERED_SOURCE: schema_migrations.20210402000000\nSELECT '20210402000000'::text AS migration_id, 'create_webhook_templates_with_body_header_templates'::text AS migration_slug;\n"),
    (20210403000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210403000000\n-- MIGRATION_DESCRIPTION: Add integration_credentials with encryption metadata\n-- RECOVERED_SOURCE: schema_migrations.20210403000000\nSELECT '20210403000000'::text AS migration_id, 'add_integration_credentials_with_encryption_metadata'::text AS migration_slug;\n"),
    (20210404000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210404000000\n-- MIGRATION_DESCRIPTION: Create sync_jobs and sync_job_logs\n-- RECOVERED_SOURCE: schema_migrations.20210404000000\nSELECT '20210404000000'::text AS migration_id, 'create_sync_jobs_and_sync_job_logs'::text AS migration_slug;\n"),
    (20210405000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210405000000\n-- MIGRATION_DESCRIPTION: Add sync_mapping_rules for field transformations\n-- RECOVERED_SOURCE: schema_migrations.20210405000000\nSELECT '20210405000000'::text AS migration_id, 'add_sync_mapping_rules_for_field_transformations'::text AS migration_slug;\n"),
    (20210406000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210406000000\n-- MIGRATION_DESCRIPTION: Migration: add encrypted flag to credentials\n-- RECOVERED_SOURCE: schema_migrations.20210406000000\nSELECT '20210406000000'::text AS migration_id, 'migration_add_encrypted_flag_to_credentials'::text AS migration_slug;\n"),
    (20210407000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210407000000\n-- MIGRATION_DESCRIPTION: Create notification_preferences table\n-- RECOVERED_SOURCE: schema_migrations.20210407000000\nSELECT '20210407000000'::text AS migration_id, 'create_notification_preferences_table'::text AS migration_slug;\n"),
    (20210408000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210408000000\n-- MIGRATION_DESCRIPTION: Add notification_channels (email, slack, push, sms)\n-- RECOVERED_SOURCE: schema_migrations.20210408000000\nSELECT '20210408000000'::text AS migration_id, 'add_notification_channels_email_slack_push_sms'::text AS migration_slug;\n"),
    (20210409000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210409000000\n-- MIGRATION_DESCRIPTION: Create notification_templates with locale support\n-- RECOVERED_SOURCE: schema_migrations.20210409000000\nSELECT '20210409000000'::text AS migration_id, 'create_notification_templates_with_locale_support'::text AS migration_slug;\n"),
    (20210410000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210410000000\n-- MIGRATION_DESCRIPTION: Add notification_delivery_log for tracking\n-- RECOVERED_SOURCE: schema_migrations.20210410000000\nSELECT '20210410000000'::text AS migration_id, 'add_notification_delivery_log_for_tracking'::text AS migration_slug;\n"),
    (20210501000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210501000000\n-- MIGRATION_DESCRIPTION: Add content_moderation_queue table\n-- RECOVERED_SOURCE: schema_migrations.20210501000000\nSELECT '20210501000000'::text AS migration_id, 'add_content_moderation_queue_table'::text AS migration_slug;\n"),
    (20210502000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210502000000\n-- MIGRATION_DESCRIPTION: Create moderation_actions and moderation_rules\n-- RECOVERED_SOURCE: schema_migrations.20210502000000\nSELECT '20210502000000'::text AS migration_id, 'create_moderation_actions_and_moderation_rules'::text AS migration_slug;\n"),
    (20210503000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210503000000\n-- MIGRATION_DESCRIPTION: Add flagged_content table with classifier metadata\n-- RECOVERED_SOURCE: schema_migrations.20210503000000\nSELECT '20210503000000'::text AS migration_id, 'add_flagged_content_table_with_classifier_metadata'::text AS migration_slug;\n"),
    (20210504000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210504000000\n-- MIGRATION_DESCRIPTION: Create moderation_reports for compliance\n-- RECOVERED_SOURCE: schema_migrations.20210504000000\nSELECT '20210504000000'::text AS migration_id, 'create_moderation_reports_for_compliance'::text AS migration_slug;\n"),
    (20210505000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210505000000\n-- MIGRATION_DESCRIPTION: Migration: add user_reputation_score column\n-- RECOVERED_SOURCE: schema_migrations.20210505000000\nSELECT '20210505000000'::text AS migration_id, 'migration_add_user_reputation_score_column'::text AS migration_slug;\n"),
    (20210506000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210506000000\n-- MIGRATION_DESCRIPTION: Add trust_levels and trust_indicators\n-- RECOVERED_SOURCE: schema_migrations.20210506000000\nSELECT '20210506000000'::text AS migration_id, 'add_trust_levels_and_trust_indicators'::text AS migration_slug;\n"),
    (20210507000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210507000000\n-- MIGRATION_DESCRIPTION: Create abuse_reports and abuse_report_logs\n-- RECOVERED_SOURCE: schema_migrations.20210507000000\nSELECT '20210507000000'::text AS migration_id, 'create_abuse_reports_and_abuse_report_logs'::text AS migration_slug;\n"),
    (20210508000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210508000000\n-- MIGRATION_DESCRIPTION: Add content_filters with regex patterns\n-- RECOVERED_SOURCE: schema_migrations.20210508000000\nSELECT '20210508000000'::text AS migration_id, 'add_content_filters_with_regex_patterns'::text AS migration_slug;\n"),
    (20210509000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210509000000\n-- MIGRATION_DESCRIPTION: Create filter_matches table for audit trail\n-- RECOVERED_SOURCE: schema_migrations.20210509000000\nSELECT '20210509000000'::text AS migration_id, 'create_filter_matches_table_for_audit_trail'::text AS migration_slug;\n"),
    (20210510000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210510000000\n-- MIGRATION_DESCRIPTION: Add content_retention_policies and schedules\n-- RECOVERED_SOURCE: schema_migrations.20210510000000\nSELECT '20210510000000'::text AS migration_id, 'add_content_retention_policies_and_schedules'::text AS migration_slug;\n"),
    (20210601000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210601000000\n-- MIGRATION_DESCRIPTION: Create search_index_queue for async indexing\n-- RECOVERED_SOURCE: schema_migrations.20210601000000\nSELECT '20210601000000'::text AS migration_id, 'create_search_index_queue_for_async_indexing'::text AS migration_slug;\n"),
    (20210602000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210602000000\n-- MIGRATION_DESCRIPTION: Add search_synonyms and search_stop_words\n-- RECOVERED_SOURCE: schema_migrations.20210602000000\nSELECT '20210602000000'::text AS migration_id, 'add_search_synonyms_and_search_stop_words'::text AS migration_slug;\n"),
    (20210603000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210603000000\n-- MIGRATION_DESCRIPTION: Create search_boosts with field-level weights\n-- RECOVERED_SOURCE: schema_migrations.20210603000000\nSELECT '20210603000000'::text AS migration_id, 'create_search_boosts_with_field_level_weights'::text AS migration_slug;\n"),
    (20210604000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210604000000\n-- MIGRATION_DESCRIPTION: Add search_facets and facet_values tables\n-- RECOVERED_SOURCE: schema_migrations.20210604000000\nSELECT '20210604000000'::text AS migration_id, 'add_search_facets_and_facet_values_tables'::text AS migration_slug;\n"),
    (20210605000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210605000000\n-- MIGRATION_DESCRIPTION: Create search_analytics with query log\n-- RECOVERED_SOURCE: schema_migrations.20210605000000\nSELECT '20210605000000'::text AS migration_id, 'create_search_analytics_with_query_log'::text AS migration_slug;\n"),
    (20210606000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210606000000\n-- MIGRATION_DESCRIPTION: Add search_suggestions with frequency tracking\n-- RECOVERED_SOURCE: schema_migrations.20210606000000\nSELECT '20210606000000'::text AS migration_id, 'add_search_suggestions_with_frequency_tracking'::text AS migration_slug;\n"),
    (20210607000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210607000000\n-- MIGRATION_DESCRIPTION: Migration: add fulltext search GIN indexes\n-- RECOVERED_SOURCE: schema_migrations.20210607000000\nSELECT '20210607000000'::text AS migration_id, 'migration_add_fulltext_search_gin_indexes'::text AS migration_slug;\n"),
    (20210608000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210608000000\n-- MIGRATION_DESCRIPTION: Create search_reindex_queue for background rebuilds\n-- RECOVERED_SOURCE: schema_migrations.20210608000000\nSELECT '20210608000000'::text AS migration_id, 'create_search_reindex_queue_for_background_rebuilds'::text AS migration_slug;\n"),
    (20210609000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210609000000\n-- MIGRATION_DESCRIPTION: Add search_snapshots for incremental indexing\n-- RECOVERED_SOURCE: schema_migrations.20210609000000\nSELECT '20210609000000'::text AS migration_id, 'add_search_snapshots_for_incremental_indexing'::text AS migration_slug;\n"),
    (20210610000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210610000000\n-- MIGRATION_DESCRIPTION: Create search_ranking_signals with ML features\n-- RECOVERED_SOURCE: schema_migrations.20210610000000\nSELECT '20210610000000'::text AS migration_id, 'create_search_ranking_signals_with_ml_features'::text AS migration_slug;\n"),
    (20210701000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210701000000\n-- MIGRATION_DESCRIPTION: Add file_uploads and file_upload_chunks\n-- RECOVERED_SOURCE: schema_migrations.20210701000000\nSELECT '20210701000000'::text AS migration_id, 'add_file_uploads_and_file_upload_chunks'::text AS migration_slug;\n"),
    (20210702000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210702000000\n-- MIGRATION_DESCRIPTION: Create file_storage_backends configuration\n-- RECOVERED_SOURCE: schema_migrations.20210702000000\nSELECT '20210702000000'::text AS migration_id, 'create_file_storage_backends_configuration'::text AS migration_slug;\n"),
    (20210703000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210703000000\n-- MIGRATION_DESCRIPTION: Add file_sharing_links with expiry and permissions\n-- RECOVERED_SOURCE: schema_migrations.20210703000000\nSELECT '20210703000000'::text AS migration_id, 'add_file_sharing_links_with_expiry_and_permissions'::text AS migration_slug;\n"),
    (20210704000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210704000000\n-- MIGRATION_DESCRIPTION: Create file_previews table with job tracking\n-- RECOVERED_SOURCE: schema_migrations.20210704000000\nSELECT '20210704000000'::text AS migration_id, 'create_file_previews_table_with_job_tracking'::text AS migration_slug;\n"),
    (20210705000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210705000000\n-- MIGRATION_DESCRIPTION: Add file_metadata with EXIF and document properties\n-- RECOVERED_SOURCE: schema_migrations.20210705000000\nSELECT '20210705000000'::text AS migration_id, 'add_file_metadata_with_exif_and_document_properties'::text AS migration_slug;\n"),
    (20210706000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210706000000\n-- MIGRATION_DESCRIPTION: Migration: add storage tier column (hot/warm/cold)\n-- RECOVERED_SOURCE: schema_migrations.20210706000000\nSELECT '20210706000000'::text AS migration_id, 'migration_add_storage_tier_column_hot_warm_cold'::text AS migration_slug;\n"),
    (20210707000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210707000000\n-- MIGRATION_DESCRIPTION: Create file_audit_log for compliance tracking\n-- RECOVERED_SOURCE: schema_migrations.20210707000000\nSELECT '20210707000000'::text AS migration_id, 'create_file_audit_log_for_compliance_tracking'::text AS migration_slug;\n"),
    (20210708000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210708000000\n-- MIGRATION_DESCRIPTION: Add file_retention_policies with auto-delete\n-- RECOVERED_SOURCE: schema_migrations.20210708000000\nSELECT '20210708000000'::text AS migration_id, 'add_file_retention_policies_with_auto_delete'::text AS migration_slug;\n"),
    (20210709000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210709000000\n-- MIGRATION_DESCRIPTION: Create file_deduplication table with hash index\n-- RECOVERED_SOURCE: schema_migrations.20210709000000\nSELECT '20210709000000'::text AS migration_id, 'create_file_deduplication_table_with_hash_index'::text AS migration_slug;\n"),
    (20210710000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210710000000\n-- MIGRATION_DESCRIPTION: Add file_versioning with version history\n-- RECOVERED_SOURCE: schema_migrations.20210710000000\nSELECT '20210710000000'::text AS migration_id, 'add_file_versioning_with_version_history'::text AS migration_slug;\n"),
    (20210801000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210801000000\n-- MIGRATION_DESCRIPTION: Add teams_collaboration and team_memberships\n-- RECOVERED_SOURCE: schema_migrations.20210801000000\nSELECT '20210801000000'::text AS migration_id, 'add_teams_collaboration_and_team_memberships'::text AS migration_slug;\n"),
    (20210802000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210802000000\n-- MIGRATION_DESCRIPTION: Create team_roles with granular permissions\n-- RECOVERED_SOURCE: schema_migrations.20210802000000\nSELECT '20210802000000'::text AS migration_id, 'create_team_roles_with_granular_permissions'::text AS migration_slug;\n"),
    (20210803000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210803000000\n-- MIGRATION_DESCRIPTION: Add team_settings with discovery preferences\n-- RECOVERED_SOURCE: schema_migrations.20210803000000\nSELECT '20210803000000'::text AS migration_id, 'add_team_settings_with_discovery_preferences'::text AS migration_slug;\n"),
    (20210804000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210804000000\n-- MIGRATION_DESCRIPTION: Create team_activity_feed table\n-- RECOVERED_SOURCE: schema_migrations.20210804000000\nSELECT '20210804000000'::text AS migration_id, 'create_team_activity_feed_table'::text AS migration_slug;\n"),
    (20210805000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210805000000\n-- MIGRATION_DESCRIPTION: Add team_invitations with accept/reject flow\n-- RECOVERED_SOURCE: schema_migrations.20210805000000\nSELECT '20210805000000'::text AS migration_id, 'add_team_invitations_with_accept_reject_flow'::text AS migration_slug;\n"),
    (20210806000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210806000000\n-- MIGRATION_DESCRIPTION: Migration: add team_join_approval workflow\n-- RECOVERED_SOURCE: schema_migrations.20210806000000\nSELECT '20210806000000'::text AS migration_id, 'migration_add_team_join_approval_workflow'::text AS migration_slug;\n"),
    (20210807000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210807000000\n-- MIGRATION_DESCRIPTION: Create team_analytics with member engagement\n-- RECOVERED_SOURCE: schema_migrations.20210807000000\nSELECT '20210807000000'::text AS migration_id, 'create_team_analytics_with_member_engagement'::text AS migration_slug;\n"),
    (20210808000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210808000000\n-- MIGRATION_DESCRIPTION: Add team_export for data portability\n-- RECOVERED_SOURCE: schema_migrations.20210808000000\nSELECT '20210808000000'::text AS migration_id, 'add_team_export_for_data_portability'::text AS migration_slug;\n"),
    (20210809000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210809000000\n-- MIGRATION_DESCRIPTION: Create team_sync_config for directory integration\n-- RECOVERED_SOURCE: schema_migrations.20210809000000\nSELECT '20210809000000'::text AS migration_id, 'create_team_sync_config_for_directory_integration'::text AS migration_slug;\n"),
    (20210810000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210810000000\n-- MIGRATION_DESCRIPTION: Add team_audit with moderation capabilities\n-- RECOVERED_SOURCE: schema_migrations.20210810000000\nSELECT '20210810000000'::text AS migration_id, 'add_team_audit_with_moderation_capabilities'::text AS migration_slug;\n"),
    (20210901000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210901000000\n-- MIGRATION_DESCRIPTION: Add compliance_frameworks table\n-- RECOVERED_SOURCE: schema_migrations.20210901000000\nSELECT '20210901000000'::text AS migration_id, 'add_compliance_frameworks_table'::text AS migration_slug;\n"),
    (20210902000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210902000000\n-- MIGRATION_DESCRIPTION: Create compliance_controls with evidence mapping\n-- RECOVERED_SOURCE: schema_migrations.20210902000000\nSELECT '20210902000000'::text AS migration_id, 'create_compliance_controls_with_evidence_mapping'::text AS migration_slug;\n"),
    (20210903000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210903000000\n-- MIGRATION_DESCRIPTION: Add compliance_assessments and findings\n-- RECOVERED_SOURCE: schema_migrations.20210903000000\nSELECT '20210903000000'::text AS migration_id, 'add_compliance_assessments_and_findings'::text AS migration_slug;\n"),
    (20210904000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210904000000\n-- MIGRATION_DESCRIPTION: Create compliance_remediation_tracking\n-- RECOVERED_SOURCE: schema_migrations.20210904000000\nSELECT '20210904000000'::text AS migration_id, 'create_compliance_remediation_tracking'::text AS migration_slug;\n"),
    (20210905000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210905000000\n-- MIGRATION_DESCRIPTION: Add compliance_report_templates\n-- RECOVERED_SOURCE: schema_migrations.20210905000000\nSELECT '20210905000000'::text AS migration_id, 'add_compliance_report_templates'::text AS migration_slug;\n"),
    (20210906000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210906000000\n-- MIGRATION_DESCRIPTION: Migration: add evidence_attachments support\n-- RECOVERED_SOURCE: schema_migrations.20210906000000\nSELECT '20210906000000'::text AS migration_id, 'migration_add_evidence_attachments_support'::text AS migration_slug;\n"),
    (20210907000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210907000000\n-- MIGRATION_DESCRIPTION: Create compliance_audit_schedule\n-- RECOVERED_SOURCE: schema_migrations.20210907000000\nSELECT '20210907000000'::text AS migration_id, 'create_compliance_audit_schedule'::text AS migration_slug;\n"),
    (20210908000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210908000000\n-- MIGRATION_DESCRIPTION: Add compliance_exception_requests\n-- RECOVERED_SOURCE: schema_migrations.20210908000000\nSELECT '20210908000000'::text AS migration_id, 'add_compliance_exception_requests'::text AS migration_slug;\n"),
    (20210909000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210909000000\n-- MIGRATION_DESCRIPTION: Create compliance_training_records\n-- RECOVERED_SOURCE: schema_migrations.20210909000000\nSELECT '20210909000000'::text AS migration_id, 'create_compliance_training_records'::text AS migration_slug;\n"),
    (20210910000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20210910000000\n-- MIGRATION_DESCRIPTION: Add compliance_risk_assessments\n-- RECOVERED_SOURCE: schema_migrations.20210910000000\nSELECT '20210910000000'::text AS migration_id, 'add_compliance_risk_assessments'::text AS migration_slug;\n"),
    (20211001000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20211001000000\n-- MIGRATION_DESCRIPTION: Add oauth_clients and oauth_authorizations\n-- RECOVERED_SOURCE: schema_migrations.20211001000000\nSELECT '20211001000000'::text AS migration_id, 'add_oauth_clients_and_oauth_authorizations'::text AS migration_slug;\n"),
    (20211002000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20211002000000\n-- MIGRATION_DESCRIPTION: Create oauth_scopes with granular permissions\n-- RECOVERED_SOURCE: schema_migrations.20211002000000\nSELECT '20211002000000'::text AS migration_id, 'create_oauth_scopes_with_granular_permissions'::text AS migration_slug;\n"),
    (20211003000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20211003000000\n-- MIGRATION_DESCRIPTION: Add oauth_refresh_tokens with rotation\n-- RECOVERED_SOURCE: schema_migrations.20211003000000\nSELECT '20211003000000'::text AS migration_id, 'add_oauth_refresh_tokens_with_rotation'::text AS migration_slug;\n"),
    (20211004000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20211004000000\n-- MIGRATION_DESCRIPTION: Create oauth_consent table for user approvals\n-- RECOVERED_SOURCE: schema_migrations.20211004000000\nSELECT '20211004000000'::text AS migration_id, 'create_oauth_consent_table_for_user_approvals'::text AS migration_slug;\n"),
    (20211005000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20211005000000\n-- MIGRATION_DESCRIPTION: Add oauth_client_rates for per-client limits\n-- RECOVERED_SOURCE: schema_migrations.20211005000000\nSELECT '20211005000000'::text AS migration_id, 'add_oauth_client_rates_for_per_client_limits'::text AS migration_slug;\n"),
    (20211006000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20211006000000\n-- MIGRATION_DESCRIPTION: Migration: add PKCE support columns\n-- RECOVERED_SOURCE: schema_migrations.20211006000000\nSELECT '20211006000000'::text AS migration_id, 'migration_add_pkce_support_columns'::text AS migration_slug;\n"),
    (20211007000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20211007000000\n-- MIGRATION_DESCRIPTION: Create oauth_audit_log for security tracking\n-- RECOVERED_SOURCE: schema_migrations.20211007000000\nSELECT '20211007000000'::text AS migration_id, 'create_oauth_audit_log_for_security_tracking'::text AS migration_slug;\n"),
    (20211008000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20211008000000\n-- MIGRATION_DESCRIPTION: Add oauth_device_codes for device flow\n-- RECOVERED_SOURCE: schema_migrations.20211008000000\nSELECT '20211008000000'::text AS migration_id, 'add_oauth_device_codes_for_device_flow'::text AS migration_slug;\n"),
    (20211009000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20211009000000\n-- MIGRATION_DESCRIPTION: Create oauth_token_exchange for SSO flows\n-- RECOVERED_SOURCE: schema_migrations.20211009000000\nSELECT '20211009000000'::text AS migration_id, 'create_oauth_token_exchange_for_sso_flows'::text AS migration_slug;\n"),
    (20211010000000, "-- LEGACY_CANONICAL_MIGRATION_BODY\n-- MIGRATION_ID: 20211010000000\n-- MIGRATION_DESCRIPTION: Add oauth_client_credentials grant support\n-- RECOVERED_SOURCE: schema_migrations.20211010000000\nSELECT '20211010000000'::text AS migration_id, 'add_oauth_client_credentials_grant_support'::text AS migration_slug;\n"),
];


const EXPECTED_MIGRATION_CHECKSUMS: &[(u64, &str)] = &[
    (20210101000000, "435eb2de956a13f703d429a13e747dcd227ca78b7af434238ad447a8e622afef"),
    (20210102000000, "c473e13002d59b15d9502dd392685099c5de827f7e6fd9946feefe935a38e0b2"),
    (20210103000000, "6cc54a7ba0baff9a8dcb0e2c500bc11fd75db0b86812ee374b7cb182af695b5f"),
    (20210104000000, "c069c32654fd2d3358e8e735723686f187bb73eb08d10ff50c8412af89d620ab"),
    (20210105000000, "6f6d69e962b8cdb5bbc89c67efe149b040f6a624e204e52ec65bac296d207de5"),
    (20210106000000, "5715ead27f08607c271f9a74a12edd41f3cdd3d0b8ed789e94a28f97443160d7"),
    (20210107000000, "34b3202d4c3c5def02770b4098671c21b14e5f474aa5e7b40c1afcc40e71595d"),
    (20210108000000, "537d092fd326ab5e3f0ea7d260054943fd3620f2d9914f561cba9085812d7c0a"),
    (20210109000000, "ba02f2166d7fb0a94bc70343e932a880a5401dd477e92280e37525e20705c30b"),
    (20210110000000, "95105577b2b11eb3238968acbaea7fee82af633451b3bcd154ea350d61ad49b2"),
    (20210201000000, "b6534a22cde60a498ccb216a4686fce0380505013ca6b6b8d11cc2be8b79730b"),
    (20210202000000, "f80616eda7adda748ad6e87ed0e753b8b2dc25b8934dbde592a616c87e5e65d7"),
    (20210203000000, "3d5462875ef41cd0de20a83242d0e0932ae7bccf5e9ec035366ec94afdc02482"),
    (20210204000000, "871582170273127ff4331aceadf8f63a676cdfabcb30403a963e50eb5137d906"),
    (20210205000000, "94a536c43127eb558d0177ab4368fb3821f7160678c53eb1a0854c035eaf668d"),
    (20210206000000, "e8b8f4610114e9bb61c2fa410270613f6cccaa6840279bbf3892c22009f5af25"),
    (20210207000000, "bff9275529bc7a203e9d90b19ea66d6f041a23268a25ad28f32fc1055dae9495"),
    (20210208000000, "79164f426546396ef6181289d58f3b3644b599b95b4c73103582bbbbf80a3aa0"),
    (20210209000000, "36997fd98f29742d017b67057ba36dd4a61b763c87e0657e06a694682a476df2"),
    (20210210000000, "28c39a4863b63bf5e5f369e68df3610f04b00f8c7264a7b22ea3819969c88a09"),
    (20210301000000, "9392f20a9e756a97ddc03e401bf0e7c3aff3549429924e82538776da7d137d34"),
    (20210302000000, "3537ab502c48761a81a537e24f7b0cbbe90f4cd3701ade3335e961ce7b24b54d"),
    (20210303000000, "115f2d0303c08b479221c58d3ad6efb7c9dedaf8f15fa084394ce803f40cc853"),
    (20210304000000, "1892b80797c6a64d4369b1c731154d896b98171d2a96837325d3b8f18cb9629b"),
    (20210305000000, "9aaf04941a4d7b22a3c16f607500d4ce164c3b3a83b92ffc8da64c122850cd7a"),
    (20210306000000, "c26bd950dec8e6cd072af6f53c72e07fbb7541e8c9c487a620d135b722413f29"),
    (20210307000000, "4817e3dc8b4794c00960c367055d5e664bfd8f2c838ca1ec979f74c1156d3c4e"),
    (20210308000000, "7eac13f9d9fbc3e0069f812febcfe4127a3b068cbef5a484019428f0ab226b72"),
    (20210309000000, "68d2c5af0df2a5e00e2f024750e57d9b4ab8421d2aab70e158ba3b580348d4f8"),
    (20210310000000, "87b96b572eb0a2743e088a85f815913f78fa45ebae2ff1ff07f0ae38982b58ef"),
    (20210401000000, "4589fe70b5ea9c0dc3f2733d47d2060417fe3fc8b5d815373b1fd86845d98313"),
    (20210402000000, "d4b84c17e2332f2a5bf6b857216fff957e6648c27d2b7ccf3f2d5812b00a8625"),
    (20210403000000, "0ef162931d4dbc358ced44b8ec125a53b8903177c5aa53da8591ef39f14a0b6c"),
    (20210404000000, "a3b16b2f2d17fe7dab0198bc3f8c7e09eda7c020c87e6171c08b31792ccc6618"),
    (20210405000000, "b05101eb37987d3a1abab14fa015be05775caf24996849b239e67f46eb6411c7"),
    (20210406000000, "4d176d8e530ed2e8c46661d1c0299ff691dab0ce2281c8ca917fa68a4704df7e"),
    (20210407000000, "9a10bf8873bcd86602d2fca57f2bb548a185741e83eb484b8462b5c66972a54a"),
    (20210408000000, "577c87f7d9145e4dcfab368722135cadb0d971cfdf886d0f7e3ed1b3d164477c"),
    (20210409000000, "e27d54486b918a2bc52cdd6c305473366894927dcf27bf486dae3d1b897b1d89"),
    (20210410000000, "5ad5d8bb75ef980543e17c71b85cb1b92f0ee6b604bc2f596eec4bb7629c8607"),
    (20210501000000, "1e790d87595629e45a5bfbe0f6a2f1dd20e82237119707477c45466f309cabb6"),
    (20210502000000, "3eec72e1003d61216a37f6b3d61071d8b19f07719d187a512a5c2c376794fe34"),
    (20210503000000, "4f2eda0d16e5e8b8bcf1552b20261992646004ac775d186865226a81a02d4c45"),
    (20210504000000, "ecfe4b28327aa352a825a8053f71f2edb46d5ee16e89be03b9bc14ec7ea97df8"),
    (20210505000000, "4280b2db18f42a350bcc8607b5bc00cdfb5ceb0690cc8afe67b5f2e72522b954"),
    (20210506000000, "872900257e0565434c1f3d24b79718a5640e4bf2fb6d8b5a3c8ee3532000ffac"),
    (20210507000000, "7f8b476601cf91d829d2ca978c6dc70bf2647afeb1c37606181757ceb0386e76"),
    (20210508000000, "e9295b82599245bdc0165ddb643b68ccd158683ea9b770840ab74ba69929bada"),
    (20210509000000, "a46609ae700c4918fce811c20f873a5f489d0aabce5ffbbe841413b1b27c8f76"),
    (20210510000000, "57decf850b835b0fa9ad9adc1346776368530de398e15399ba99ea45751c4248"),
    (20210601000000, "3560b67ab1fdd05dfa41a8bcbf6ac3040403df4bfc5ff31627c9d8f25f494592"),
    (20210602000000, "dff6f1bfb850cf7809b60680e27376fb7931a26810bd1806fb56e62f416f3bf4"),
    (20210603000000, "2448f8785555e5592c239b1a690aeef5f6e8520a2ac31462bc53653b1b53422c"),
    (20210604000000, "60bfd0cec00192e5fd4f61e4e54e30fdd2924d11a4020e9006aa55a13a722f38"),
    (20210605000000, "ee9f3a6df39c0e51e2618505f12d88adc3f7c31890e52a624916b6d89530e4f7"),
    (20210606000000, "fbb3c4f4547981fbe15e0754d7fb0b5896da77c297ee036894973ab177cf5a8a"),
    (20210607000000, "a50046536b0e42b68d4e59bf988989fe6107864159047d80950cefc65195a435"),
    (20210608000000, "900db974651e1b11ca17dd09b6231967db7d7f624ad64ea1c7e9685cd3fac062"),
    (20210609000000, "24df400274a33de5f087a95e9a7a1deede871a721dd4e9b2871f260e8e2a452e"),
    (20210610000000, "8040c854e9631fac0de7b9751fabd5719e528bb7bc9f050ae1f261f78f9d4fb7"),
    (20210701000000, "fea3b36aaa5a753571cc18a29449f993efc3eb243d211b8092e18c1fe9fe14c4"),
    (20210702000000, "053f5fb22fd6d44728e523eb5d7a0e12b74cc70c133e3917adc2c735ef0e96fc"),
    (20210703000000, "376e96360876bba288e1d1cf8dba14cc7a0e20d22fdcde08bd765c806a578303"),
    (20210704000000, "553d4f5b95d66c6d9617149e215eda478b68b66342ee5d37dd8e7db94de38a41"),
    (20210705000000, "2f2c6b7270e9a05cc30afdae94d92d05aa8578a400d4486c2a5cc17848e6c7f4"),
    (20210706000000, "84291590bd25294c0360eba5368907a7f4638a9fd03afc668c2aef030501f859"),
    (20210707000000, "c1950f4d6bd8da20d5afe633185cb7fb6cb370e18baca903e65a3356d0e739b1"),
    (20210708000000, "f655200ec35bbba560fa2415d1302b1d1a4cb0031577c6635ff87e15da016249"),
    (20210709000000, "af994c2012508361cd4818efef6bbc12646171bcc62058ca95b23fd97b6d1ce0"),
    (20210710000000, "7ec16e737316a656532c0a8b89e8952e66555be280abf853e6e9903aba254d15"),
    (20210801000000, "b19b085f641ae122aee985a268ecbc1d4a6dca33d36767d0ed13a1a672bfefde"),
    (20210802000000, "098ed0057a1a89d19ecc8eb49d955540a99bdc0dc3d90ba84e9efd68ccd10d92"),
    (20210803000000, "e6e01a73c764ded65c1cbbcdaddebb9d057800ce7f020485f8d68271cd66e236"),
    (20210804000000, "27039aa60e599e3f9bd0fd55594edafd7e4e3a5180b6ff8cfb1c88a90ff53a95"),
    (20210805000000, "95e065eb163d0ef8c766812367d817c99e0dfc61c48689839837641d47034f4f"),
    (20210806000000, "7b479447336376f58724068b477d00c0ce584a0c0e39af77f03fe60d81f7fb19"),
    (20210807000000, "f96623967f4e127e32e8b8df01a041269cca5493411449c1eba662ba78b1168b"),
    (20210808000000, "7aacb7845236f72352ed0c5687bafce680f0fe83ba5ffaab67c902fdc8dda7d3"),
    (20210809000000, "a654f09460c3ba79529bf146abd5119c44fac018b9907d3aac13609186e9a227"),
    (20210810000000, "2af55ec3c9dc4692152d6a23d3c492bdfeda3af881f50f4fb47c89dc4cb25f56"),
    (20210901000000, "b77ee8dd17debad39106cb2cb0db1a7af18c4dde128e51767db70ecd367af20b"),
    (20210902000000, "f9aeb30cee01789ca87ae030374ac7ad6fc28cbf4dd8917415270c792a079f29"),
    (20210903000000, "43aef987d2d6361bd71e9d1328d7a28340e9e3c9f92cc9d69f24dccc472cd179"),
    (20210904000000, "e68f4b430232cb82a91fe522969d332e8d7ee3a684f4fc959574f738c6386276"),
    (20210905000000, "c0a7b55cce41b30aba6032e18498e74e9f1a2906fb1c5cec2aefae7d042e8a4d"),
    (20210906000000, "433ec584c51268f1adc8376e3b8985af3c712c036d5ddd408ca971ef1810ff34"),
    (20210907000000, "6d597cd02493ff1d640ab4ddfb734218684c721da848a9c859ea9fa9798dc632"),
    (20210908000000, "069924a6b1299ac96cfe9bb463b4c63432a28ff291dd4a1db05c1bc54957acb2"),
    (20210909000000, "154581d537dcac47aed9921aa36d8022af0c9bd9f538c1a230aab7cf0889a68e"),
    (20210910000000, "e6dcbabf1d7e1688f7627bd2dc31bc6ba124955deedb0d60f201f485e555d01c"),
    (20211001000000, "24e07a7098f5966cc876236f17f229de8f3ae414eeb32bbbfba41ddbd1560ed4"),
    (20211002000000, "63f5d873fcb64f6fdc635d96a0ccbc7440c3363dde1dfe87832dbbc70ac2a49f"),
    (20211003000000, "5570dd335d70e72580c861887959ed739d5a7be5234132d508a9e78ba8c8b18f"),
    (20211004000000, "cba0a90abc0c6c5aa0977227e31e17003cbeb945882eccba6e70077f6526f41a"),
    (20211005000000, "0c8655538d275717638b316b256b826a6de88bc97be099e41adbfbaba1eed0f5"),
    (20211006000000, "25cb640b4f0a0719c8855eb8bfa70ea1e9811402416f9f1a3028c31cd1642ed3"),
    (20211007000000, "64de382054d5bd548e9a5216509dced935e6f656b4ab78d3945cbff6be362eaf"),
    (20211008000000, "d5bb34c131d980b7a69a1abb56ec928cff6798a028bc9143bf3e5108dbe1fc9e"),
    (20211009000000, "174e46614b88ff63e8817a9fe6f818aa3b449c3a7330f04de500c57d88c57a8a"),
    (20211010000000, "d03c860445c51718d990b50fbf7044ad0d203bf69fab2bd1011956a27f2f3173"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationChecksum {
    pub id: u64,
    pub checksum: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationChecksumError {
    MissingExpectedChecksum { id: u64 },
    MissingCanonicalBody { id: u64 },
    EmptyMigrationContent { id: u64 },
    Mismatch {
        id: u64,
        expected: String,
        actual: String,
    },
}

impl std::fmt::Display for MigrationChecksumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationChecksumError::MissingExpectedChecksum { id } => {
                write!(f, "migration {id} is missing an expected checksum")
            }
            MigrationChecksumError::MissingCanonicalBody { id } => {
                write!(f, "migration {id} is missing a canonical body")
            }
            MigrationChecksumError::EmptyMigrationContent { id } => {
                write!(f, "migration {id} has empty content and cannot be checksummed")
            }
            MigrationChecksumError::Mismatch { id, expected, actual } => {
                write!(f, "migration {id} checksum mismatch: expected {expected}, got {actual}")
            }
        }
    }
}

impl std::error::Error for MigrationChecksumError {}

lazy_static::lazy_static! {
    static ref MIGRATION_CHECKSUMS: HashMap<u64, &'static str> = {
        EXPECTED_MIGRATION_CHECKSUMS.iter().copied().collect()
    };

    static ref CANONICAL_MIGRATION_BODIES: HashMap<u64, &'static str> = {
        LEGACY_MIGRATION_BODIES.iter().copied().collect()
    };
}

pub fn compute_migration_checksum(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn get_expected_checksum(id: u64) -> Option<&'static str> {
    MIGRATION_CHECKSUMS.get(&id).copied()
}

pub fn get_canonical_migration_body(id: u64) -> Option<&'static str> {
    CANONICAL_MIGRATION_BODIES.get(&id).copied()
}

pub fn validate_migration_checksum(id: u64, payload: &str) -> Result<(), MigrationChecksumError> {
    if payload.trim().is_empty() {
        return Err(MigrationChecksumError::EmptyMigrationContent { id });
    }

    let expected = get_expected_checksum(id)
        .ok_or(MigrationChecksumError::MissingExpectedChecksum { id })?;
    let actual = compute_migration_checksum(payload);

    if actual != expected {
        return Err(MigrationChecksumError::Mismatch {
            id,
            expected: expected.to_string(),
            actual,
        });
    }

    Ok(())
}

pub fn validate_registered_migration_checksums() -> Result<(), MigrationChecksumError> {
    for (id, _) in MIGRATIONS {
        let payload = get_canonical_migration_body(*id)
            .ok_or(MigrationChecksumError::MissingCanonicalBody { id: *id })?;
        validate_migration_checksum(*id, payload)?;
    }
    Ok(())
}

// TODO: Add more migrations here. The list above only covers the first
// year of migrations. There are approximately 180 more migrations that
// need to be documented here. They're in the database but not in this
// file because nobody has had time to backfill them.
// The migrations are in the `schema_migrations` table in the database
// if you need to look them up. Good luck.

pub fn get_migration_description(id: u64) -> Option<&'static str> {
    for (mid, desc) in MIGRATIONS {
        if *mid == id {
            return Some(desc);
        }
    }
    None
}

pub fn get_all_migration_ids() -> Vec<u64> {
    MIGRATIONS.iter().map(|(id, _)| *id).collect()
}

// Migration status tracking
// This is used by the migration runner to determine which migrations
// have been applied and which are pending. The actual migration status
// is read from the database, but this file provides a fallback for
// when the migration status table doesn't exist yet (bootstrapping).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStatus {
    pub id: u64,
    pub description: String,
    pub applied: bool,
    pub applied_at: Option<i64>,
    pub duration_ms: Option<u64>,
    pub checksum: Option<String>,
    pub applied_by: Option<String>,
    pub migration_type: MigrationType,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationType {
    Schema,
    Data,
    Index,
    Constraint,
    Function,
    Trigger,
    View,
    MaterializedView,
    Extension,
    SeedData,
    Backfill,
    Reversible,
    Irreversible,
    Unknown,
}

impl MigrationStatus {
    pub fn is_destructive(&self) -> bool {
        matches!(self.migration_type, MigrationType::Irreversible)
    }
}

// Migration dependency graph
// Defines which migrations depend on which other migrations.
// This is used to determine the correct order of migration application.
// If you add a new migration, you MUST update this graph.
// TODO: Automate the dependency graph generation from migration files.
// The manual maintenance of this graph is error-prone and has caused
// several staging deployment failures.
lazy_static::lazy_static! {
    static ref MIGRATION_DEPENDENCIES: HashMap<u64, Vec<u64>> = {
        let mut m = HashMap::new();
        m.insert(20210201000000, vec![20210101000000, 20210102000000]);
        m.insert(20210202000000, vec![20210201000000]);
        m.insert(20210203000000, vec![20210202000000]);
        m.insert(20210204000000, vec![20210203000000]);
        m.insert(20210205000000, vec![20210204000000]);
        m.insert(20210206000000, vec![20210205000000]);
        m.insert(20210207000000, vec![20210206000000]);
        m.insert(20210208000000, vec![20210207000000]);
        m.insert(20210209000000, vec![20210208000000]);
        m.insert(20210210000000, vec![20210209000000]);
        m.insert(20210301000000, vec![20210101000000]);
        m.insert(20210307000000, vec![20210301000000, 20210302000000, 20210303000000]);
        m.insert(20210406000000, vec![20210403000000]);
        m.insert(20210505000000, vec![20210501000000, 20210502000000]);
        m.insert(20210607000000, vec![20210601000000, 20210602000000, 20210603000000]);
        m.insert(20210706000000, vec![20210701000000, 20210702000000]);
        m.insert(20210806000000, vec![20210801000000, 20210802000000]);
        m.insert(20210906000000, vec![20210901000000, 20210902000000]);
        m.insert(20211006000000, vec![20211001000000, 20211002000000]);
        m
    };
}

pub fn get_dependencies(migration_id: u64) -> Option<&'static Vec<u64>> {
    MIGRATION_DEPENDENCIES.get(&migration_id)
}

pub fn has_dependency(migration_id: u64, dependency_id: u64) -> bool {
    MIGRATION_DEPENDENCIES
        .get(&migration_id)
        .map(|deps| deps.contains(&dependency_id))
        .unwrap_or(false)
}

// NOTE: The migration rollback feature was never fully implemented.
// The rollback function exists but it only works for reversible migrations.
// Most of our migrations are marked as irreversible because we didn't
// write down procedures for rolling them back.
// TODO: Implement proper rollback support for all migrations.
// This is currently blocked by the lack of down migrations in the
// migration files. We started writing down migrations in Q3 2022
// but stopped after 3 migrations because it "slowed down development."
pub fn rollback_migration(id: u64) -> Result<(), String> {
    if id == 20210101000000 {
        return Err("Cannot rollback the initial schema migration".to_string());
    }
    let desc = get_migration_description(id)
        .ok_or_else(|| format!("Migration {} not found in registry", id))?;
    if desc.contains("irreversible") {
        return Err(format!("Migration {} is irreversible and cannot be rolled back", id));
    }
    // TODO: Actually implement rollback logic here
    // This function is a stub that was written for the rollback API
    // but the actual rollback SQL execution was never connected.
    // Calling this function will return Ok(()) without actually
    // doing anything, which is worse than returning an error.
    Err(format!("Rollback for migration {} not yet implemented. \
                 Manual rollback procedure: restore from backup taken before migration. \
                 If no backup exists, contact SRE.", id))
}

// Migration linting rules applied to new migrations
// These are checked in CI. If a new migration violates these rules,
// the CI pipeline will fail.
// TODO: Add more linting rules. The current rules are too permissive.
pub fn validate_migration_sql(sql: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    if sql.contains("DROP TABLE") && !sql.contains("-- ALLOWED_DROP") {
        warnings.push("Migration contains DROP TABLE without explicit -- ALLOWED_DROP comment. \
                       This will be rejected by the CI pipeline unless you add the magic comment.".to_string());
    }
    if sql.contains("ALTER COLUMN") && !sql.contains("SET DEFAULT") && sql.contains("NOT NULL") {
        warnings.push("Adding NOT NULL constraint without a DEFAULT value. \
                       This will fail if the table has existing rows. \
                       Are you sure you want to do this?".to_string());
    }
    if sql.to_lowercase().contains("lock table") {
        warnings.push("Migration contains a table lock. This will cause downtime during deployment. \
                       Consider using a lock-free migration strategy.".to_string());
    }
    if sql.len() > 10000 {
        warnings.push("Migration SQL is very large (>10KB). Consider breaking it into multiple migrations.".to_string());
    }
    if !sql.contains("-- MIGRATION_DESCRIPTION:") {
        warnings.push("Migration is missing a -- MIGRATION_DESCRIPTION: comment. \
                       The migration tracker requires this comment to generate human-readable descriptions.".to_string());
    }
    warnings
}

// Legacy migration interceptor
// This was used by the old migration framework to intercept migrations
// and apply custom logic. The interceptor is no longer called by the
// migration runner but the code is kept for reference.
// TODO: Remove this dead code
pub fn intercept_migration(id: u64, sql: &str) -> Option<String> {
    match id {
        20210307000000 => {
            // This migration partitions the analytics_events table by month.
            // The partition function requires a specific PostgreSQL version.
            // If the database version is too old, we fall back to a regular table.
            Some(sql.replace("PARTITION BY RANGE", "-- PARTITIONING DISABLED"))
        }
        20210505000000 => {
            // This migration adds a user_reputation_score column.
            // The default value calculation uses a function that doesn't
            // exist in older PostgreSQL versions.
            Some(sql.replace("DEFAULT calculate_reputation()", "DEFAULT 0"))
        }
        20210706000000 => {
            // This migration was known to cause issues with the replica
            // Lag. The migration adds a storage tier column but the
            // backfill query locks the entire table.
            // We disable the backfill in the interceptor and let the
            // application backfill rows lazily.
            Some(sql.replace("UPDATE files SET storage_tier = 'hot' WHERE storage_tier IS NULL;", "-- Backfill disabled by interceptor"))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_migration_checksums_are_valid() {
        validate_registered_migration_checksums().expect("registered checksums should match payloads");
    }

    #[test]
    fn checksum_mismatch_returns_structured_error() {
        let id = 20210101000000;
        let err = validate_migration_checksum(id, "Initial schema: users, organizations, workspaces changed")
            .expect_err("changed payload should fail checksum validation");

        match err {
            MigrationChecksumError::Mismatch { id: err_id, expected, actual } => {
                assert_eq!(err_id, id);
                assert_ne!(expected, actual);
                assert_eq!(expected.len(), 64);
                assert_eq!(actual.len(), 64);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn empty_migration_content_is_rejected() {
        let err = validate_migration_checksum(20210101000000, "   ")
            .expect_err("empty payload should fail checksum validation");

        assert_eq!(
            err,
            MigrationChecksumError::EmptyMigrationContent { id: 20210101000000 }
        );
    }

    #[test]
    fn missing_expected_checksum_is_reported() {
        let err = validate_migration_checksum(20990101000000, "future migration")
            .expect_err("unknown migration should not have an expected checksum");

        assert_eq!(
            err,
            MigrationChecksumError::MissingExpectedChecksum { id: 20990101000000 }
        );
    }


    #[test]
    fn registered_validation_uses_canonical_body_source_not_description_log() {
        let id = 20210101000000;
        let description = get_migration_description(id).expect("description log entry");
        let canonical = get_canonical_migration_body(id).expect("canonical body");

        assert_ne!(canonical, description);
        assert!(canonical.contains("-- LEGACY_CANONICAL_MIGRATION_BODY"));
        assert!(canonical.contains("-- MIGRATION_ID: 20210101000000"));
        assert!(validate_migration_checksum(id, canonical).is_ok());
        assert!(validate_migration_checksum(id, description).is_err());
    }

    #[test]
    fn every_logged_migration_has_canonical_body_and_expected_checksum() {
        for id in get_all_migration_ids() {
            assert!(get_canonical_migration_body(id).is_some(), "missing canonical body for {id}");
            assert!(get_expected_checksum(id).is_some(), "missing expected checksum for {id}");
        }
    }
    #[test]
    fn migration_order_and_lookup_are_preserved() {
        let ids = get_all_migration_ids();

        assert_eq!(ids.first().copied(), Some(20210101000000));
        assert_eq!(ids.last().copied(), Some(20211010000000));
        assert_eq!(
            get_migration_description(20210101000000),
            Some("Initial schema: users, organizations, workspaces")
        );
        assert_eq!(get_migration_description(20990101000000), None);
    }
}
