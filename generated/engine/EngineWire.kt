// Generated from the Rust engine contracts.
package org.viptv.engine.wire

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
enum class Availability {
    @SerialName("Available") AVAILABLE,
    @SerialName("NeedsConfiguration") NEEDSCONFIGURATION,
    @SerialName("NeedsAuthorization") NEEDSAUTHORIZATION,
    @SerialName("TemporarilyUnavailable") TEMPORARILYUNAVAILABLE,
    @SerialName("Unsupported") UNSUPPORTED,
}

@Serializable
enum class Capability {
    @SerialName("Discovery") DISCOVERY,
    @SerialName("Search") SEARCH,
    @SerialName("Metadata") METADATA,
    @SerialName("Sources") SOURCES,
    @SerialName("Subtitles") SUBTITLES,
    @SerialName("Tracking") TRACKING,
    @SerialName("Authorization") AUTHORIZATION,
    @SerialName("Mapping") MAPPING,
    @SerialName("LiveTv") LIVETV,
}

@Serializable
data class CapabilityState(
    val capability: Capability,
    val availability: Availability,
    val message: String? = null,
)

@Serializable
data class Card(
    val identity: MediaIdentity,
    val provider_id: String,
    val title: String,
    val subtitle: String? = null,
    val description: String? = null,
    val poster: String? = null,
    val landscape: String? = null,
    val logo: String? = null,
    val progress: Double? = null,
    val badges: List<String>,
)

@Serializable
data class CatalogPage(
    val cards: List<Card>,
    val next_cursor: String? = null,
    val complete: Boolean,
)

@Serializable
data class Choice(
    val value: String,
    val label: String,
)

@Serializable
enum class ControlKind {
    @SerialName("Text") TEXT,
    @SerialName("SingleChoice") SINGLECHOICE,
    @SerialName("MultiChoice") MULTICHOICE,
    @SerialName("Toggle") TOGGLE,
    @SerialName("Number") NUMBER,
    @SerialName("Date") DATE,
}

@Serializable
data class DetailSnapshot(
    val item: Card,
    val episodes: List<Card>,
    val next_cursor: String? = null,
)

@Serializable
data class EpisodeCoordinates(
    val order: String,
    val season: Long? = null,
    val episode: Long,
)

@Serializable
enum class ExecutionMode {
    @SerialName("Local") LOCAL,
    @SerialName("Server") SERVER,
}

@Serializable
data class ExternalId(
    val namespace: String,
    val value: String,
)

@Serializable
data class HistoryEntry(
    val card: Card,
    val updated_at_millis: Long,
    val completed_at_millis: Long? = null,
)

@Serializable
data class HomeRow(
    val id: String,
    val title: String,
    val provider_id: String,
    val state: RowState,
    val cards: List<Card>,
    val next_cursor: String? = null,
    val message: String? = null,
)

@Serializable
data class HomeSnapshot(
    val scope: Scope,
    val revision: Long,
    val hero: Card? = null,
    val rows: List<HomeRow>,
    val complete: Boolean,
)

@Serializable
data class IntegrationConnection(
    val id: String,
    val plugin_id: String,
    val account_id: String,
    val private_profile_id: String? = null,
    val name: String,
    val enabled: Boolean,
    val revision: Long,
    val capabilities: List<Capability>,
)

@Serializable
enum class MappingConfidence {
    @SerialName("Explicit") EXPLICIT,
    @SerialName("Candidate") CANDIDATE,
}

@Serializable
data class MappingEvidence(
    val source: String,
    val target: ExternalId,
    val confidence: MappingConfidence,
    val episode: EpisodeCoordinates? = null,
    val version: String,
)

@Serializable
data class MediaIdentity(
    val id: String,
    val kind: String,
    val ids: List<ExternalId>,
    val episode: EpisodeCoordinates? = null,
    val edition: String? = null,
)

@Serializable
data class PlaybackRecord(
    val identity: MediaIdentity,
    val provider_id: String,
    val title: String,
    val poster: String? = null,
    val landscape: String? = null,
    val position_millis: Long,
    val duration_millis: Long,
    val updated_at_millis: Long,
    val completed_at_millis: Long? = null,
)

@Serializable
data class PluginDescriptor(
    val id: String,
    val name: String,
    val contract_version: Long,
    val locations: List<ExecutionMode>,
    val capabilities: List<Capability>,
)

@Serializable
data class ProfileAssignment(
    val connection_id: String,
    val enabled: List<Capability>,
)

@Serializable
data class ProfileConfiguration(
    val scope: Scope,
    val mode: ExecutionMode,
    val name: String,
    val avatar_id: String? = null,
    val browsing_plugin: String,
    val assignments: List<ProfileAssignment>,
)

@Serializable
data class QueryControl(
    val id: String,
    val label: String,
    val kind: ControlKind,
    val required: Boolean,
    val choices: List<Choice>,
    val minimum: Double? = null,
    val maximum: Double? = null,
)

@Serializable
data class QueryValue(
    val id: String,
    val values: List<String>,
)

@Serializable
enum class RowState {
    @SerialName("Pending") PENDING,
    @SerialName("Ready") READY,
    @SerialName("Empty") EMPTY,
    @SerialName("Failed") FAILED,
}

@Serializable
data class Scope(
    val connection_id: String,
    val account_id: String,
    val profile_id: String,
    val revision: Long,
)

@Serializable
data class SourceCandidate(
    val id: String,
    val provider_id: String,
    val label: String,
    val detail: String? = null,
)

@Serializable
data class SubtitleCandidate(
    val id: String,
    val provider_id: String,
    val language: String,
    val label: String,
)

@Serializable
data class SubtitleProvider(
    val provider_id: String,
    val state: RowState,
    val message: String? = null,
)

@Serializable
data class SubtitleSnapshot(
    val scope: Scope,
    val candidates: List<SubtitleCandidate>,
    val providers: List<SubtitleProvider>,
    val complete: Boolean,
)

