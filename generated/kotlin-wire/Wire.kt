// Generated from Rust Facet registry; do not edit.
package org.viptv.core.wire
import kotlinx.serialization.*
import kotlinx.serialization.json.*

object CoreJson { val codec = Json { ignoreUnknownKeys = true; explicitNulls = false }
 inline fun <reified T> decode(value: String): T = codec.decodeFromString(value)
 inline fun <reified T> encode(value: T): String = codec.encodeToString(value)
}
@Serializable enum class MediaKind { @SerialName("movie") MOVIE, @SerialName("series") SERIES, @SerialName("live") LIVE, @SerialName("episode") EPISODE }
@Serializable enum class Phase { @SerialName("Starting") STARTING, @SerialName("Restoring") RESTORING, @SerialName("Checking") CHECKING, @SerialName("Selecting") SELECTING, @SerialName("Ready") READY, @SerialName("Profiles") PROFILES, @SerialName("Pairing") PAIRING, @SerialName("Error") ERROR }
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
    val `type`: MediaKind,
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
@Serializable data class DiscoverPage(
    val `items`: List<MediaItem> = emptyList(),
    val `hasMore`: Boolean,
    val `nextSkip`: Double? = null
)
@Serializable data class Identity(
    val `account`: Account,
    val `profiles`: List<Profile> = emptyList(),
    val `profileId`: String? = null,
    val `restricted`: Boolean,
    val `profileSetupRequired`: Boolean
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
@Serializable data class PlaybackSession(
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
    val `subtitlesSupported`: Boolean
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
@Serializable data class SourcePresentation(
    val `title`: String,
    val `body`: String
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
