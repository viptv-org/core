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
@Serializable data class ViewModel(
    val `phase`: Phase,
    val `identity`: Identity? = null,
    val `selectedProfileId`: String? = null,
    val `error`: String? = null,
    val `errorStatus`: Int? = null
)
