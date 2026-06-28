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

const EXPECTED_MIGRATION_CHECKSUMS: &[(u64, &str)] = &[
    (20210101000000, "7af5f160895bf42c15553fde0a55ffb66b8967c1d91698c527aa43a46d526c74"),
    (20210102000000, "a38a0ec90013cb0807291dcfde29ea4efd29fa5d7016403ca65591de948dbe05"),
    (20210103000000, "d2aad2897b5fe9c802c397fc9fbef25eb9a8e6d80a2ee8abc6a07f186bf09f35"),
    (20210104000000, "514314fd20eccc3b9a2fe8a64a7adfd41bc1e89e3e4e248c4b1e5ac1b6c5588a"),
    (20210105000000, "20bc96d5adc7f1112123eb11e978ddc319330883510c8ba33ebcd459c8e9cc6a"),
    (20210106000000, "55e97e78c6154fd58e47a6c107b432c68dacd3babb5185c059b2a48ccf4ae66d"),
    (20210107000000, "89df17fdac9cecdad6cd536d0a1ce7c64a3bcf37a027119286783c4cf4214f82"),
    (20210108000000, "4b43a2989c942ec618e2e6fb0159ddc25607c85bc3c1e4b615f0febb45bd8664"),
    (20210109000000, "43b27eb025c890db405a5959cf7313f972dad39cd905e7271d6ebb61111296dc"),
    (20210110000000, "2a73e0540a2f4f49998cd80f777329d55d76835669dbf7548fb0f6730688130e"),
    (20210201000000, "e5e9966ce85d5a9f453cec4fd604ab3692c4d841c0dd776e556d5d0c9253ed57"),
    (20210202000000, "446e70b7be3b31a573d81937651182f65f60e308f3cea36734f8462ed8a3e336"),
    (20210203000000, "c95188165294867b8060dd656192e4e5d0128d4641e1c33d7d94bf16ee963b53"),
    (20210204000000, "a1564ddcc17d2deb8e83e55a0bca9efbdec7b5bf7be8a9e14d923de29d39a019"),
    (20210205000000, "2cb48dce2565857d6cb0527d61ba259fe5a4665631ab8bbd9b703c658315a07c"),
    (20210206000000, "70497a739250e6fdbb5367c8b4f66eea1d94fb1ea2294d79f6a16d5fc24e200b"),
    (20210207000000, "14260472eba34aafcc9b38845af5f9f67f4a4284fe3640683bc4ee6113b6aef6"),
    (20210208000000, "effa397bb940c26d073736c538dde3f9e5f3c480af52bcfe1adbbb5b68b25e04"),
    (20210209000000, "763e302bfd7f89905d72d8f6089741d71f278f3b593aed1815f2304cd195d367"),
    (20210210000000, "f6a1cfd242c446af8c98262e284aae85211fb4a24598eccb11b3973e691fbae5"),
    (20210301000000, "ddf250bc914ccc48aeee9edc8489b3265b4a21bc9fbd3bd8e5e824985659da90"),
    (20210302000000, "d24f0cca732b66a0c36a02fa10c6b5b4966720069e8bf51be55a728849d7c8c3"),
    (20210303000000, "e7ff57065161a6199b4db66e5619ff3f6c9fcdb5a27085737f60b90f83d6246b"),
    (20210304000000, "eca106f87875fdb9b74f4b9722e06daaab97feae6e478c578d8483ffa63925de"),
    (20210305000000, "4fdb3f07c3284a72a6e4a65461a2c67b132d48cbbae9febe5b479b0788567dd2"),
    (20210306000000, "42962d5dd75c1ae82f88c6bd60a4b638c4e523b982e6cbe1374c4a865d3bb157"),
    (20210307000000, "244c8de15d68a9ca1f07510d10609985a0f17c48f1a869493a2eb09794f8815e"),
    (20210308000000, "3058d66a2fb7ed8ebea7b70a460531e90ea8d35cab622037ceabf4afa40cd332"),
    (20210309000000, "185189845e4ce27d583f4ee3b1c00d9b858f7c2c2123b2195fab17ef6617df18"),
    (20210310000000, "47bb693bbeb86220e19a52e8e3482584b66e377d7982ef234ed410e3d11c582a"),
    (20210401000000, "8699c1ca4ddb88a0afea5aa40f53ba7317cfd66aa5a867b44563eba8ab7fefd7"),
    (20210402000000, "337a8c933cbb422990db353d5ffdbfebc2fddbd8447b0ef5d498dd940db36ecb"),
    (20210403000000, "8f9bf6ea02adaf6d2a9c17e06527f46d05a7a7b777c8b7fbed7887c56a48cd0d"),
    (20210404000000, "5682442d0e173bb6f8a21ab27abf757e984574e6d66e26a5a05ef834b81721e1"),
    (20210405000000, "cb6f0a730c8c10fac5e5de264874c67c00654fe69cba630a418fdc40c89616de"),
    (20210406000000, "20f6c67fd09f30d449f59a2492ab292f7b6afa16e918364781b05270375776dc"),
    (20210407000000, "d83efa2ad043d7cade5c932e756dd8c1a9c54788d5251845a965be31351daf3b"),
    (20210408000000, "8094f0ff1ecb4c20023d38757225b38ec4d4ec90eae8c6cb974b15ccb5e99a1c"),
    (20210409000000, "75dbe4cc7a6d7ca452a66b7be86268771e480baa259fb37408584c7b657d7910"),
    (20210410000000, "79ca07be34bb30a44e069d15f4c5aa46071681d7b5def7b5b56fd1bd1d1c7d0f"),
    (20210501000000, "93b2b5abc2641cf5917919059f4743f380a41fcc915cd68e0925ff150d8c9628"),
    (20210502000000, "24d176e22a47a2bc3551b67c896d63b2dc8d71aaa82a88aa70ab89b153d9c24b"),
    (20210503000000, "c3f1abc1c0b95bbac04065aca11368e70ce2babfa0f26b0ec00d9c6165ba4605"),
    (20210504000000, "75d10295461aa048f3f094398f76202071f233de35be6cbcb1af54311053470d"),
    (20210505000000, "2e99e78eeca6c73cd45940fd5183345d547563d02ca369a73382d5db7ac66422"),
    (20210506000000, "a3027c2eaaea7cf62d1f5cb81cd2f097e9f3770d5b48c7d5f888eb3a9b6259fd"),
    (20210507000000, "b7395797cc0dc5a76edd127099713ae259dca0c886f477c2dfe405812cd44ac3"),
    (20210508000000, "fdf4b0e3caccc9dcff36706b463c40ed213c0558427a618e997398e95d47b2de"),
    (20210509000000, "b725bda543707ecad217f87380a46e443088eb94e0524bb5c32e8ec4a31c4659"),
    (20210510000000, "d348eac83ee848664c4145a81f9767f1aca8b7bfa20eea3b3060989c58f87690"),
    (20210601000000, "b7554f5dc5d8eabc2ac807294828dcd52bfd5bf2894058084741ff4a191dc38e"),
    (20210602000000, "ea710f6202217a4678897bcaa76a9d7975a420c9865c1816b1bbbb140c195efc"),
    (20210603000000, "9d9de1031fae1f49554741834a24ddcc95911b32085b9d29507f59f1db956afd"),
    (20210604000000, "054a4afda56b32aa457bfe3900feb68dc1dbdd65e70992bf44b50ad08bf3f070"),
    (20210605000000, "7fed9e7af70ac76802283d715f4a4d0cadf63661bbcc40e285f800b47d0a48eb"),
    (20210606000000, "a40972fed41ffc263acbcbb560a8b990eead52280fbc9f95501445401de7bcf5"),
    (20210607000000, "e287bf71bffdfd4d5380e3658ee83557447325d375c26cf71a856096736c89ed"),
    (20210608000000, "06c0f95f180e0128c386de24570df5244ef0f3c25679866cc8927b23f105cb50"),
    (20210609000000, "322139d2a9bae6b0387b363ce190701c9e90eb2dedac80304760969895e7fc10"),
    (20210610000000, "871622f1999fa495df8c8cb14e48be08426c5502c1c0556e32c1f131502bdd21"),
    (20210701000000, "d74c28643ee6d36c7d2fff30c96904747e518256eff8cfc9c7cdd91d40f98cfc"),
    (20210702000000, "436a85cda8e6039746661ec22ffddc724dd28e02741076ac68192f7c4c24d454"),
    (20210703000000, "672cf2d2ff7408a22ee66d8a32c737e4d775cb1a792811ce0ede58c8bc4791db"),
    (20210704000000, "8a0faf55d5c0a6b2c413a575cfb44d39ea268fb4fe2e57bf4c77e8e697bdbb66"),
    (20210705000000, "986e70d7f8ace3bfc5eb377d02d9054c756b23fb3e9648aa1c22eafae339f478"),
    (20210706000000, "8c519a6ef52d5ef8903c14faaa4e1281521ee36461a9de163fdc6931a6c2c8ab"),
    (20210707000000, "e88656d0678f32c87aa3d454edd5d955143b1bb543c5c2e1725295d07b231d28"),
    (20210708000000, "936183fc3186379ee664957a01268bdbf14e2272c9572149d8b469ab4c7cbc3b"),
    (20210709000000, "8af1e815ba221956f1ec9af38345ec5566b4790ff42323daaee87459920009c6"),
    (20210710000000, "3a2f228b81177fb753110a1f9cb6447a55235896e34bc76aeccd5691b4b378c1"),
    (20210801000000, "b9ce1bb2cdb0cea56c534e699e0e62b602867aa4940c8487551e83feae81513e"),
    (20210802000000, "80ed18bfda4c8f26375c83032f1bc1b2a5440e0863340e5fb548c51c89a04fd7"),
    (20210803000000, "862a5fd6a7790ba6b81da0aeddea424643c878af804f56f021490be65665d79e"),
    (20210804000000, "a817feed219bb1dd85f1f3befe9851c15b109986fcadbb46e0369c8de24611ef"),
    (20210805000000, "3ff1a1b1a2f099ceb5239696070efe04a0cf389d1a3a04858732dd0083423823"),
    (20210806000000, "02966eb1dff644e0bd853279d0e318c3f5cda4085fdbe81ff09a9ef282df435a"),
    (20210807000000, "1f4eea806bbd723f91f5c3602fa2f4aca621f75a762cdce503e05b605c024e89"),
    (20210808000000, "07f93e5af04c49977a0a7e912236a181b6b0082a432538f8ddfae3604a578b9a"),
    (20210809000000, "e009a862bd2c10e6505f43772b384bf262e475cd22ae3930eca65281fadbb0fe"),
    (20210810000000, "bcf69d8ce3037fdb69b41255ad32fa6291c3ab23de593552bbd101d529a23507"),
    (20210901000000, "2ef5043ad745a85b37c4216bf7b299a550207c9c06fd827a90ef1afc5df92412"),
    (20210902000000, "c5b6681f5a136cc2e0769486dafc2cf5d746887e7b23e440da69d3d1d0869278"),
    (20210903000000, "c84e648411fcefa89775e6f5cbc4df794a6352176c5beb92cf750cef975f2b9d"),
    (20210904000000, "fc50df80be5c23803292285db1def48547a03335e9db5dc7f49d27c3c5b2d361"),
    (20210905000000, "90c570aaa419eb75ca6f64e3d2f41837067fcc6fff5a36c844495b660d6a6b20"),
    (20210906000000, "f640ba90e4e710e47f5fb28f625b5437b8d3523862aaccf617780942a990620d"),
    (20210907000000, "113d9293df2ce6ffcb7dbcabcb55d50b6e186d7c4dc53569387cc075ab7628a6"),
    (20210908000000, "ca3635fa567640faf8c688d0bd847b3085e066adb8ecb9b9016fdd01d916382a"),
    (20210909000000, "22e4242ffe2aee0d7a5c1b634f478c99ada7e5ca7ebb9af6759226e5dc00a35c"),
    (20210910000000, "567970489b9b64762aa5bdb20ae7d8d8a486d64268e32132e153f9eb5624e61f"),
    (20211001000000, "6c6c03b61b6d3022a658d12e02aafa96c54c3c4b56b37211f7e4a2105558398a"),
    (20211002000000, "324efc8c1dfda6c6dc95deaf3f2ed8de9723cf18d2380610408e26e209d78e70"),
    (20211003000000, "d65fd7516b90fc8da2ae8ad441f8cd33aaf87dfe75e5777c7f0e793198902f93"),
    (20211004000000, "cfcc341e6b7aad5b66d9b92f3bfbd1dea6a7bdaa1bc842fac6ef7818d2aed6dd"),
    (20211005000000, "f1a59ee84eaf2f34dd5e4b6960c97dd1b965900e350ad81b1d27bb3552af3bec"),
    (20211006000000, "2ee430ef661cc8dcd4ca4933680f6498f1293e0070f192f7b33a389aa7f63d4e"),
    (20211007000000, "207038b608a8d0e6c82544f9c5c52c5521498a4c749197e6d1301da9304f6d1a"),
    (20211008000000, "2a246bf290d4dc550c8915910a90950323c374bd38b66381da98127d7a38f12c"),
    (20211009000000, "e0694500eb9268371358c4e33df33f54ca7f54dc418403ac90145ae950631c09"),
    (20211010000000, "d7a36cf38f006d427c00b7417396fca0172725c7f93c98781eaace4d98dffa04"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationChecksum {
    pub id: u64,
    pub checksum: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationChecksumError {
    MissingExpectedChecksum { id: u64 },
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
}

pub fn compute_migration_checksum(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn get_expected_checksum(id: u64) -> Option<&'static str> {
    MIGRATION_CHECKSUMS.get(&id).copied()
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
    for (id, payload) in MIGRATIONS {
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
