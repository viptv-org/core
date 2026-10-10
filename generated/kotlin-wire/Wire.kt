// Generated from Rust Facet registry; do not edit.
package org.viptv.core.wire
import kotlinx.serialization.*
import kotlinx.serialization.json.*

object CoreJson { val codec = Json { ignoreUnknownKeys = true; explicitNulls = false }
 inline fun <reified T> decode(value: String): T = codec.decodeFromString(value)
 inline fun <reified T> encode(value: T): String = codec.encodeToString(value)
}
@Serializable enum class CountdownAction { @SerialName("Begin") BEGIN, @SerialName("Advance") ADVANCE, @SerialName("Cancel") CANCEL }
@Serializable enum class ForegroundAuthorityDecision { @SerialName("Valid") VALID, @SerialName("Revoked") REVOKED, @SerialName("ProfileUnavailable") PROFILEUNAVAILABLE }
@Serializable enum class HomeRevisionDecision { @SerialName("Unchanged") UNCHANGED, @SerialName("Refresh") REFRESH, @SerialName("Refreshed") REFRESHED, @SerialName("RetryLater") RETRYLATER, @SerialName("Unsupported") UNSUPPORTED, @SerialName("ScopeLost") SCOPELOST }
@Serializable enum class LivePageValidationDecision { @SerialName("valid") VALID, @SerialName("catalog_changed") CATALOG_CHANGED, @SerialName("invalid") INVALID }
@Serializable enum class MediaKind { @SerialName("movie") MOVIE, @SerialName("series") SERIES, @SerialName("live") LIVE, @SerialName("episode") EPISODE }
@Serializable enum class NativeTorrentControlOperation { @SerialName("start") START, @SerialName("poll") POLL, @SerialName("heartbeat") HEARTBEAT }
@Serializable enum class NativeTorrentNegotiationDecision { @SerialName("rejectStale") REJECTSTALE, @SerialName("authRecovery") AUTHRECOVERY, @SerialName("legacy") LEGACY, @SerialName("advertise") ADVERTISE }
@Serializable enum class NativeTorrentRecoveryAction { @SerialName("retry") RETRY, @SerialName("chooseSource") CHOOSESOURCE, @SerialName("back") BACK }
@Serializable enum class NativeTorrentRecoveryDecision { @SerialName("waitForRetirement") WAITFORRETIREMENT, @SerialName("authRecovery") AUTHRECOVERY, @SerialName("chooseSource") CHOOSESOURCE, @SerialName("back") BACK, @SerialName("ordinaryRetry") ORDINARYRETRY, @SerialName("forceGatewayRetry") FORCEGATEWAYRETRY, @SerialName("nativeRetry") NATIVERETRY }
@Serializable enum class Phase { @SerialName("Starting") STARTING, @SerialName("Restoring") RESTORING, @SerialName("Checking") CHECKING, @SerialName("Selecting") SELECTING, @SerialName("Ready") READY, @SerialName("Profiles") PROFILES, @SerialName("Pairing") PAIRING, @SerialName("Error") ERROR }
@Serializable enum class PlaybackConversion { @SerialName("auto") AUTO, @SerialName("audio") AUDIO, @SerialName("video") VIDEO, @SerialName("audio_video") AUDIO_VIDEO }
@Serializable enum class PlaybackDeliveryKind { @SerialName("direct") DIRECT, @SerialName("gateway") GATEWAY }
@Serializable enum class PlaybackLeaseDecision { @SerialName("invalid") INVALID, @SerialName("terminal") TERMINAL, @SerialName("expired") EXPIRED, @SerialName("pending") PENDING, @SerialName("ready") READY }
@Serializable enum class PlaybackLeaseStatus { @SerialName("starting") STARTING, @SerialName("ready") READY, @SerialName("failed") FAILED, @SerialName("expired") EXPIRED, @SerialName("released") RELEASED }
@Serializable enum class PlaybackPlatform { @SerialName("android") ANDROID, @SerialName("android_tv") ANDROID_TV, @SerialName("desktop") DESKTOP, @SerialName("web") WEB, @SerialName("tizen") TIZEN, @SerialName("webos") WEBOS, @SerialName("roku") ROKU, @SerialName("vizio") VIZIO }
@Serializable enum class PreviewAction { @SerialName("Start") START, @SerialName("Adopt") ADOPT, @SerialName("Update") UPDATE, @SerialName("Result") RESULT }
@Serializable enum class PreviewDecision { @SerialName("Retain") RETAIN, @SerialName("BeginSettled") BEGINSETTLED, @SerialName("BeginImmediate") BEGINIMMEDIATE, @SerialName("Accept") ACCEPT, @SerialName("Reject") REJECT, @SerialName("Cancelled") CANCELLED, @SerialName("Failed") FAILED, @SerialName("Ready") READY }
@Serializable enum class PreviewRoute { @SerialName("Details") DETAILS, @SerialName("Sources") SOURCES, @SerialName("Player") PLAYER, @SerialName("Other") OTHER }
@Serializable enum class VizioControllerOutputKind { @SerialName("request") REQUEST, @SerialName("complete") COMPLETE, @SerialName("error") ERROR }
@Serializable enum class VizioFailureKind { @SerialName("invalidConfig") INVALIDCONFIG, @SerialName("invalidInput") INVALIDINPUT, @SerialName("authentication") AUTHENTICATION, @SerialName("invalidParameter") INVALIDPARAMETER, @SerialName("endpointNotFound") ENDPOINTNOTFOUND, @SerialName("busy") BUSY, @SerialName("transport") TRANSPORT, @SerialName("invalidResponse") INVALIDRESPONSE, @SerialName("httpStatus") HTTPSTATUS }
@Serializable enum class VizioHttpMethod { @SerialName("GET") GET, @SerialName("PUT") PUT }
@Serializable enum class VizioRemoteAction { @SerialName("KEYPRESS") KEYPRESS, @SerialName("KEYDOWN") KEYDOWN, @SerialName("KEYUP") KEYUP }
@Serializable enum class VizioRemoteKey { @SerialName("SEEK_FWD") SEEK_FWD, @SerialName("SEEK_BACK") SEEK_BACK, @SerialName("PAUSE") PAUSE, @SerialName("PLAY") PLAY, @SerialName("DOWN") DOWN, @SerialName("LEFT") LEFT, @SerialName("OK") OK, @SerialName("RIGHT") RIGHT, @SerialName("UP") UP, @SerialName("BACK") BACK, @SerialName("SMARTCAST") SMARTCAST, @SerialName("CC_TOGGLE") CC_TOGGLE, @SerialName("INFO") INFO, @SerialName("MENU") MENU, @SerialName("HOME") HOME, @SerialName("VOL_DOWN") VOL_DOWN, @SerialName("VOL_UP") VOL_UP, @SerialName("MUTE_OFF") MUTE_OFF, @SerialName("MUTE_ON") MUTE_ON, @SerialName("MUTE_TOGGLE") MUTE_TOGGLE, @SerialName("PIC_MODE") PIC_MODE, @SerialName("PIC_SIZE") PIC_SIZE, @SerialName("INPUT_NEXT") INPUT_NEXT, @SerialName("CH_DOWN") CH_DOWN, @SerialName("CH_UP") CH_UP, @SerialName("CH_PREV") CH_PREV, @SerialName("EXIT") EXIT, @SerialName("POW_OFF") POW_OFF, @SerialName("POW_ON") POW_ON, @SerialName("POW_TOGGLE") POW_TOGGLE }
@Serializable enum class VizioTransportSupport { @SerialName("native") NATIVE, @SerialName("unavailable") UNAVAILABLE }
@Serializable data class Account(
    val `id`: String,
    val `username`: String,
    val `name`: String,
    val `role`: String
)
@Serializable data class ApiRequest(
    val `method`: String,
    val `path`: String,
    val `body`: Map<String, JsonElement>? = null
)
@Serializable data class CardPresentation(
    val `image`: String? = null,
    val `imageRole`: String,
    val `title`: String,
    val `subtitle`: String,
    val `progress`: Double? = null,
    val `primaryAction`: String,
    val `primaryActionLabel`: String
)
@Serializable data class Catalog(
    val `id`: String,
    val `name`: String,
    val `type`: String,
    val `addonName`: String? = null,
    val `addonId`: Double? = null,
    val `addonKey`: String? = null,
    val `supportsSearch`: Boolean,
    val `supportsSkip`: Boolean,
    val `extras`: List<CatalogExtra> = emptyList(),
    val `genres`: List<String> = emptyList(),
    val `raw`: Map<String, JsonElement> = emptyMap()
)
@Serializable data class CatalogExtra(
    val `name`: String,
    val `required`: Boolean,
    val `options`: List<String> = emptyList(),
    val `defaultValue`: String? = null,
    val `optionsLimit`: Double? = null
)
@Serializable data class CountdownDecision(
    val `remainingMillis`: Long,
    val `seconds`: Long,
    val `active`: Boolean,
    val `done`: Boolean
)
@Serializable data class CountdownInput(
    val `action`: CountdownAction,
    val `remainingMillis`: Long,
    val `active`: Boolean,
    val `elapsedMillis`: Long,
    val `progressing`: Boolean,
    val `scopeMatches`: Boolean
)
@Serializable data class DiscoverPage(
    val `items`: List<MediaItem> = emptyList(),
    val `unsupportedCount`: Long? = null,
    val `hasMore`: Boolean,
    val `nextSkip`: Double? = null
)
@Serializable data class DiscoverPolicyProjection(
    val `group`: String,
    val `groupLabel`: String,
    val `firstCatalogIndex`: Long? = null,
    val `defaults`: Map<String, String> = emptyMap()
)
@Serializable data class EpisodeWatching(
    val `watching`: Boolean,
    val `progress`: Double
)
@Serializable data class ForegroundAuthorityInput(
    val `expected`: Identity,
    val `current`: Identity,
    val `profileId`: String? = null
)
@Serializable data class HomeActions(
    val `canManage`: Boolean,
    val `managePrevious`: Boolean,
    val `canResume`: Boolean,
    val `hasResolvedNext`: Boolean,
    val `opensQueueManage`: Boolean,
    val `opensSourcesFromHero`: Boolean,
    val `cardPrimaryAction`: String,
    val `heroPrimaryAction`: String,
    val `heroPrimaryActionLabel`: String,
    val `showHeroProgress`: Boolean
)
@Serializable data class HomeRevisionInput(
    val `scopeValid`: Boolean,
    val `observedRevision`: String? = null,
    val `renderedRevision`: String? = null,
    val `refreshSucceeded`: Boolean? = null
)
@Serializable data class Identity(
    val `account`: Account,
    val `profiles`: List<Profile> = emptyList(),
    val `profileId`: String? = null,
    val `restricted`: Boolean,
    val `profileSetupRequired`: Boolean
)
@Serializable data class LiveCatalogCategories(
    val `catalogId`: String? = null,
    val `generation`: String? = null,
    val `items`: List<LiveCatalogCategory> = emptyList(),
    val `nextCursor`: String? = null,
    val `previousCursor`: String? = null
)
@Serializable data class LiveCatalogCategory(
    val `id`: String,
    val `name`: String
)
@Serializable data class LiveCatalogPage(
    val `catalogId`: String? = null,
    val `generation`: String? = null,
    val `items`: List<MediaItem> = emptyList(),
    val `nextCursor`: String? = null,
    val `previousCursor`: String? = null
)
@Serializable data class LivePageValidationFacts(
    val `catalogId`: String? = null,
    val `generation`: String? = null,
    val `ids`: List<String> = emptyList(),
    val `names`: List<String> = emptyList(),
    val `categories`: Boolean,
    val `nextCursor`: String? = null,
    val `previousCursor`: String? = null,
    val `requestedCatalogId`: String? = null,
    val `limit`: Long,
    val `checkSnapshot`: Boolean,
    val `snapshotCatalogId`: String? = null,
    val `snapshotGeneration`: String? = null,
    val `knownIds`: List<String> = emptyList(),
    val `cursor`: String? = null,
    val `previous`: Boolean,
    val `extendingWindow`: Boolean
)
@Serializable data class MediaItem(
    val `id`: String,
    val `type`: MediaKind,
    val `name`: String,
    val `title`: String,
    val `poster`: String? = null,
    val `background`: String? = null,
    val `thumbnail`: String? = null,
    val `titleLogo`: String? = null,
    val `imdbRating`: String? = null,
    val `credits`: String? = null,
    val `posterShape`: String? = null,
    val `updatedAtMillis`: Double? = null,
    val `releasedAtMillis`: Double? = null,
    val `episodes`: List<MediaItem> = emptyList(),
    val `description`: String? = null,
    val `year`: Double? = null,
    val `runtime`: String? = null,
    val `genres`: List<String> = emptyList(),
    val `position`: Double? = null,
    val `duration`: Double? = null,
    val `watched`: Boolean? = null,
    val `resumeActive`: Boolean? = null,
    val `watchDateKnown`: Boolean? = null,
    val `completionOnly`: Boolean? = null,
    val `season`: Double? = null,
    val `episode`: Double? = null,
    val `episodeTitle`: String? = null,
    val `seriesId`: String? = null,
    val `queueStatus`: String? = null,
    val `previousEpisode`: MediaItem? = null,
    val `sourceAddonId`: String? = null,
    val `sourceName`: String? = null,
    val `sourceFingerprint`: String? = null,
    val `sourceBingeGroup`: String? = null,
    val `sourceReleaseGroup`: String? = null,
    val `sourceQuality`: String? = null,
    val `sourceAudio`: String? = null,
    val `raw`: Map<String, JsonElement> = emptyMap()
)
@Serializable data class MediaPresentation(
    val `heroImage`: String? = null,
    val `posterImage`: String? = null,
    val `episodeImage`: String? = null,
    val `titleLogo`: String? = null,
    val `title`: String,
    val `episodeLabel`: String,
    val `progress`: Double,
    val `primaryAction`: String,
    val `primaryActionLabel`: String,
    val `resumeEligible`: Boolean,
    val `canAutoNext`: Boolean
)
@Serializable data class MediaSource(
    val `provider`: String? = null,
    val `description`: String? = null,
    val `sourceFingerprint`: String? = null,
    val `id`: String,
    val `name`: String,
    val `title`: String? = null,
    val `filename`: String? = null,
    val `sourceAddonId`: String? = null,
    val `sourceName`: String? = null,
    val `quality`: String? = null,
    val `audio`: String? = null,
    val `raw`: Map<String, JsonElement> = emptyMap()
)
@Serializable data class MediaTrack(
    val `inputIndex`: Double,
    val `codec`: String? = null,
    val `language`: String? = null,
    val `languageStatus`: String,
    val `title`: String,
    val `selected`: Boolean,
    val `supported`: Boolean,
    val `selectable`: Boolean
)
@Serializable data class MetadataTarget(
    val `type`: String,
    val `id`: String
)
@Serializable data class NativeTorrentCapability(
    val `version`: Long,
    val `networkPolicy`: String
)
@Serializable data class NativeTorrentClock(
    val `scope`: String,
    val `generation`: Long,
    val `nowMillis`: Long,
    val `trustedWallUpperUnixMillis`: Long? = null,
    val `backendRevalidated`: Boolean,
    val `suspendAware`: Boolean
)
@Serializable data class NativeTorrentContext(
    val `origin`: String,
    val `scope`: String,
    val `generation`: Long,
    val `qualified`: Boolean,
    val `negotiated`: Boolean,
    val `vod`: Boolean,
    val `request`: PlaybackV2Request
)
@Serializable data class NativeTorrentNegotiationFacts(
    val `platform`: PlaybackPlatform,
    val `qualified`: Boolean,
    val `scopeMatches`: Boolean,
    val `status`: Int? = null,
    val `authorizationRefused`: Boolean,
    val `body`: String
) { override fun toString(): String = "NativeTorrentNegotiationFacts(<redacted>)" }
@Serializable data class NativeTorrentObservation(
    val `scope`: String,
    val `generation`: Long,
    val `sequence`: Long,
    val `operation`: NativeTorrentControlOperation,
    val `receivedAtMillis`: Long,
    val `roundTripMillis`: Long,
    val `uncertaintyMillis`: Long? = null,
    val `maxUncertaintyMillis`: Long,
    val `trustedWallUpperUnixMillis`: Long? = null,
    val `suspendAware`: Boolean
)
@Serializable data class NativeTorrentRecoveryFacts(
    val `admitted`: Boolean,
    val `authorityRetired`: Boolean,
    val `authorizationRefused`: Boolean,
    val `selectionRefused`: Boolean,
    val `action`: NativeTorrentRecoveryAction
)
@Serializable data class NativeTorrentState(
    val `status`: String,
    val `deadlineMillis`: Long? = null,
    val `expiresAtUnixMillis`: Long? = null,
    val `position`: Double? = null,
    val `audioLanguage`: String? = null,
    val `subtitleLanguage`: String? = null,
    val `subtitlesEnabled`: Boolean? = null,
    val `error`: String? = null
)
@Serializable data class PhonePresentation(
    val `shelfHeading`: String,
    val `cardContext`: String,
    val `contentTypeLabel`: String
)
@Serializable data class PlaybackAuthorityBudget(
    val `remainingMillis`: Long,
    val `delayMillis`: Long
)
@Serializable data class PlaybackAuthorityFacts(
    val `expiresAtMillis`: Long,
    val `nowMillis`: Long,
    val `elapsedMillis`: Long,
    val `observationCapMillis`: Long,
    val `waitMillis`: Long
)
@Serializable data class PlaybackAuthorization(
    val `cookie`: String? = null,
    val `userAgent`: String? = null,
    val `headers`: Map<String, String>? = null
)
@Serializable data class PlaybackClient(
    val `platform`: PlaybackPlatform,
    val `canPlayDirect`: Boolean,
    val `maxWidth`: Long,
    val `maxHeight`: Long,
    val `videoCodecs`: List<String> = emptyList(),
    val `audioCodecs`: List<String> = emptyList(),
    val `nativeTorrent`: NativeTorrentCapability? = null
)
@Serializable data class PlaybackDeliveryFacts(
    val `directDelivery`: Boolean,
    val `canPlayDirect`: Boolean,
    val `forceGateway`: Boolean,
    val `automaticConversion`: Boolean
)
@Serializable data class PlaybackFailureDecision(
    val `retryRenewal`: Boolean,
    val `reconcileAndRelease`: Boolean
)
@Serializable data class PlaybackFailureFacts(
    val `gatewayError`: Boolean,
    val `status`: Int,
    val `invalidResponse`: Boolean,
    val `ioError`: Boolean,
    val `hasLeaseId`: Boolean
)
@Serializable data class PlaybackLease(
    val `id`: String,
    val `status`: PlaybackLeaseStatus,
    val `expiresAt`: Double,
    val `renewAfterSeconds`: Long,
    val `session`: PlaybackSession? = null,
    val `errorCode`: String? = null,
    val `error`: String? = null
)
@Serializable data class PlaybackLeaseFacts(
    val `expectedId`: String,
    val `actualId`: String,
    val `status`: String,
    val `hasSession`: Boolean,
    val `expiresAtMillis`: Double,
    val `nowMillis`: Long,
    val `heartbeat`: Boolean,
    val `sameDeliveryUrl`: Boolean,
    val `sameDeliveryKind`: Boolean
)
@Serializable data class PlaybackPauseDecision(
    val `usesAnchor`: Boolean,
    val `replaceOnResume`: Boolean,
    val `anchorAfterOpenMillis`: Long? = null
)
@Serializable data class PlaybackPauseFacts(
    val `deliveryMode`: String,
    val `live`: Boolean,
    val `anchorMillis`: Long? = null,
    val `launchPositionMillis`: Long,
    val `playWhenReady`: Boolean
)
@Serializable data class PlaybackProtocol(
    val `version`: Long,
    val `nativeTorrentVersions`: List<Long> = emptyList()
)
@Serializable data class PlaybackRecoveryFacts(
    val `serverManaged`: Boolean,
    val `networkFailure`: Boolean,
    val `alreadyAttempted`: Boolean
)
@Serializable data class PlaybackSeekFacts(
    val `currentMillis`: Long,
    val `deltaMillis`: Long,
    val `durationMillis`: Long? = null,
    val `rangeStartMillis`: Long? = null,
    val `rangeEndMillis`: Long? = null
)
@Serializable data class PlaybackSession(
    val `deliveryKind`: PlaybackDeliveryKind? = null,
    val `preferredAudioLanguage`: String? = null,
    val `preferredSubtitleLanguage`: String? = null,
    val `headers`: Map<String, String> = emptyMap(),
    val `id`: String,
    val `url`: String,
    val `format`: String,
    val `mode`: String,
    val `videoMode`: String,
    val `audioMode`: String,
    val `position`: Double,
    val `live`: Boolean,
    val `duration`: Double,
    val `audioTracks`: List<MediaTrack> = emptyList(),
    val `subtitleTracks`: List<MediaTrack> = emptyList(),
    val `subtitlesSupported`: Boolean,
    val `authorization`: PlaybackAuthorization? = null
)
@Serializable data class PlaybackTimelineFacts(
    val `deliveryMode`: String,
    val `launchPositionMillis`: Long,
    val `segmentPositionMillis`: Long,
    val `titleOffsetMillis`: Long,
    val `titlePositionMillis`: Long,
    val `nativeDurationMillis`: Long? = null,
    val `titleDurationMillis`: Long? = null,
    val `pauseAnchorMillis`: Long? = null,
    val `playerError`: Boolean,
    val `trustedPositionMillis`: Long
)
@Serializable data class PlaybackTimelineProjection(
    val `launchOffsetMillis`: Long,
    val `positionMillis`: Long,
    val `segmentPositionMillis`: Long,
    val `durationMillis`: Long? = null,
    val `updateTrustedPosition`: Boolean
)
@Serializable data class PlaybackV2Request(
    val `conversion`: PlaybackConversion,
    val `requestId`: String,
    val `streamId`: String,
    val `client`: PlaybackClient,
    val `position`: Double,
    val `forceGateway`: Boolean,
    val `audioTrack`: Long? = null,
    val `subtitleTrack`: Long? = null,
    val `audioLanguage`: String? = null,
    val `preferredAudioLanguage`: String? = null,
    val `preferredSubtitleLanguage`: String? = null,
    val `subtitlesOff`: Boolean
)
@Serializable data class PreviewInput(
    val `action`: PreviewAction,
    val `requestedKey`: String,
    val `activeKey`: String? = null,
    val `running`: Boolean,
    val `hasSources`: Boolean,
    val `done`: Boolean,
    val `failed`: Boolean,
    val `ownerMatches`: Boolean,
    val `elapsedMillis`: Long? = null,
    val `reuseBudgetMillis`: Long? = null
)
@Serializable data class PreviewScopeDecision(
    val `key`: String? = null,
    val `keep`: Boolean
)
@Serializable data class PreviewScopeInput(
    val `profileId`: String? = null,
    val `mediaType`: String,
    val `mediaId`: String,
    val `hasEpisode`: Boolean,
    val `activeKey`: String? = null,
    val `route`: PreviewRoute,
    val `releasing`: Boolean
)
@Serializable data class Profile(
    val `raw`: Map<String, JsonElement> = emptyMap(),
    val `id`: String,
    val `name`: String,
    val `avatar`: String? = null,
    val `primary`: Boolean? = null,
    val `avatarStyle`: String? = null,
    val `avatarChoice`: Double? = null,
    val `kid`: Boolean? = null,
    val `setupComplete`: Boolean? = null
)
@Serializable data class Session(
    val `sessionId`: String,
    val `accountId`: String,
    val `profileId`: String? = null,
    val `accessToken`: String,
    val `refreshToken`: String,
    val `expiresIn`: Double
)
@Serializable data class SourceFailure(
    val `source`: String,
    val `code`: String? = null,
    val `message`: String
)
@Serializable data class SourcePresentation(
    val `title`: String,
    val `body`: String,
    val `providerKey`: String,
    val `providerLabel`: String
)
@Serializable data class SourceProducerOutcome(
    val `sourceId`: String,
    val `label`: String,
    val `errorCode`: String? = null,
    val `errorMessage`: String? = null
)
@Serializable data class SourceRank(
    val `rank`: Double,
    val `likely`: Boolean,
    val `best`: Boolean
)
@Serializable data class SourceRanks(
    val `ranks`: List<SourceRank> = emptyList(),
    val `orderedIndices`: List<Long> = emptyList()
)
@Serializable data class SourcesPollState(
    val `after`: Double,
    val `sources`: List<MediaSource> = emptyList(),
    val `polls`: Long,
    val `errors`: List<SourceFailure>? = null,
    val `producers`: List<SourceProducerOutcome> = emptyList()
)
@Serializable data class SourcesPollStep(
    val `state`: SourcesPollState,
    val `sources`: List<MediaSource> = emptyList(),
    val `done`: Boolean,
    val `producers`: List<SourceProducerOutcome> = emptyList()
)
@Serializable data class UpNextGateDecision(
    val `attemptedKey`: String? = null,
    val `resumeAwaitingKey`: String? = null,
    val `start`: Boolean
)
@Serializable data class UpNextGateInput(
    val `mediaKey`: String,
    val `attemptedKey`: String? = null,
    val `resumeAwaitingKey`: String? = null,
    val `ended`: Boolean,
    val `eligible`: Boolean,
    val `continuationBusy`: Boolean,
    val `blocked`: Boolean
)
@Serializable data class UpNextPlaybackDecision(
    val `attemptedKey`: String? = null,
    val `resumeAwaitingKey`: String? = null
)
@Serializable data class UpNextPlaybackInput(
    val `mediaKey`: String,
    val `previousKey`: String? = null,
    val `attemptedKey`: String? = null,
    val `resumeAwaitingKey`: String? = null,
    val `explicitResume`: Boolean,
    val `positionMillis`: Long,
    val `durationMillis`: Long? = null
)
@Serializable data class ViewModel(
    val `phase`: Phase,
    val `identity`: Identity? = null,
    val `selectedProfileId`: String? = null,
    val `error`: String? = null,
    val `errorStatus`: Int? = null
)
@Serializable data class VizioAppConfig(
    val `appId`: String,
    val `nameSpace`: Int,
    val `message`: String? = null
)
@Serializable data class VizioControllerOutput(
    val `kind`: VizioControllerOutputKind,
    val `requestId`: Long? = null,
    val `request`: VizioRequest? = null,
    val `result`: Map<String, JsonElement>? = null,
    val `error`: VizioFailure? = null,
    val `credentialChanged`: Boolean
)
@Serializable data class VizioDiscoveryCandidate(
    val `host`: String,
    val `port`: Int
)
@Serializable data class VizioFailure(
    val `kind`: VizioFailureKind,
    val `message`: String,
    val `retryable`: Boolean,
    val `protocolStatus`: String? = null
)
@Serializable data class VizioInputInfo(
    val `cname`: String,
    val `name`: String,
    val `metaName`: String,
    val `current`: Boolean,
    val `hashValue`: Long? = null
)
@Serializable data class VizioPairingChallenge(
    val `challengeType`: Int,
    val `token`: Long
)
@Serializable data class VizioPlatformSupport(
    val `protocolAvailable`: Boolean,
    val `transport`: VizioTransportSupport,
    val `reason`: String
)
@Serializable data class VizioProtocolResponse(
    val `status`: String,
    val `detail`: String,
    val `raw`: Map<String, JsonElement> = emptyMap()
)
@Serializable data class VizioRemoteEvent(
    val `codeSet`: Int,
    val `code`: Int,
    val `action`: VizioRemoteAction
)
@Serializable data class VizioRequest(
    val `method`: VizioHttpMethod,
    val `url`: String,
    val `headers`: Map<String, String> = emptyMap(),
    val `body`: Map<String, JsonElement>? = null,
    val `timeoutMillis`: Long,
    val `maxResponseBytes`: Long
)
