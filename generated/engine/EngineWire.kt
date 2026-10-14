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
    val category: String? = null,
    val metadata: CardMetadata? = null,
    val provider_id: String,
    val title: String,
    val subtitle: String? = null,
    val description: String? = null,
    val poster: String? = null,
    val landscape: String? = null,
    val logo: String? = null,
    val progress: Double? = null,
    val badges: List<String>,
    val parent_identity: MediaIdentity? = null,
    val added_at_millis: Long? = null,
)

@Serializable
data class CardMetadata(
    val year: Long? = null,
    val runtime_minutes: Long? = null,
    val genres: List<String> = emptyList(),
    val rating: Double? = null,
    val rating_votes: Long? = null,
    val certification: String? = null,
    val alternate_titles: List<String> = emptyList(),
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
    val recommendations: List<Card> = emptyList(),
    val episodes: List<Card>,
    val next_cursor: String? = null,
    val seasons: List<EpisodeSeason> = emptyList(),
)

@Serializable
data class EpisodeCoordinates(
    val order: String,
    val season: Long? = null,
    val episode: Long,
)

@Serializable
data class EpisodeSeason(
    val season: Long? = null,
    val episode_count: Long,
    val offset: Long,
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
    val featured: List<Card> = emptyList(),
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
data class LibraryImportStatus(
    val provider_id: String,
    val running: Boolean,
    val checked_at_millis: Long? = null,
    val library_revision: Long? = null,
    val added_titles: Long,
    val imported_history: Long,
    val preserved_local: Long,
    val unresolved: Long,
    val availability: Availability,
    val refresh_after_millis: Long? = null,
)

@Serializable
data class LibraryProvenance(
    val provider_id: String,
    val external_identity: MediaIdentity,
    val imported_at_millis: Long,
    val source_updated_at_millis: Long? = null,
    val link_revision: Long? = null,
)

@Serializable
data class LibraryQuery(
    val source: String,
    val offset: Long,
    val limit: Long,
    val sort: String,
    val show: String,
    val type: String,
    val status: String? = null,
    val genre: String? = null,
    val rated: String? = null,
    val added: String? = null,
    val episodes_left_max: Long? = null,
)

@Serializable
data class LibrarySnapshot(
    val scope: Scope,
    val revision: Long,
    val query: LibraryQuery,
    val cards: List<Card>,
    val next_cursor: String? = null,
    val complete: Boolean,
    val controls: List<QueryControl> = emptyList(),
    val sources: List<LibrarySource> = emptyList(),
    val statistics: List<LibraryStatistics> = emptyList(),
    val availability: Availability,
    val reason: String? = null,
    val refresh_after_millis: Long? = null,
)

@Serializable
data class LibrarySource(
    val id: String,
    val label: String,
    val availability: Availability,
    val reason: String? = null,
)

@Serializable
data class LibraryStatistics(
    val identity: MediaIdentity,
    val provenance: List<LibraryProvenance> = emptyList(),
    val watched: Boolean,
    val last_watched_millis: Long? = null,
    val times_watched: Long? = null,
    val user_rating: Double? = null,
    val watched_episode_count: Long? = null,
    val total_episode_count: Long? = null,
    val aired_episode_count: Long? = null,
    val has_unwatched_aired_episodes: Boolean? = null,
    val status: String? = null,
    val next_episode: EpisodeCoordinates? = null,
    val next_air_millis: Long? = null,
)

@Serializable
data class LiveChannel(
    val id: String,
    val connection_id: String,
    val card: Card,
    val group_id: String? = null,
    val group_name: String? = null,
    val group_ids: List<String> = emptyList(),
)

@Serializable
data class LiveGroup(
    val id: String,
    val label: String,
)

@Serializable
data class LiveGuide(
    val channel_id: String,
    val programs: List<LiveProgram>,
    val availability: Availability,
    val reason: String? = null,
)

@Serializable
data class LivePage(
    val channels: List<LiveChannel>,
    val groups: List<LiveGroup>,
    val connections: List<IntegrationConnection>,
    val next_cursor: String? = null,
    val complete: Boolean,
    val availability: Availability,
    val reason: String? = null,
)

@Serializable
data class LiveProgram(
    val id: String,
    val title: String,
    val description: String? = null,
    val start_millis: Long,
    val end_millis: Long,
)

@Serializable
data class LiveQuery(
    val connection_id: String? = null,
    val group_id: String? = null,
    val search: String? = null,
    val offset: Long,
    val limit: Long? = null,
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
    val times_watched: Long? = null,
    val presentation: Card? = null,
    val identity: MediaIdentity,
    val provider_id: String,
    val title: String,
    val poster: String? = null,
    val landscape: String? = null,
    val position_millis: Long,
    val duration_millis: Long,
    val updated_at_millis: Long,
    val completed_at_millis: Long? = null,
    val parent_identity: MediaIdentity? = null,
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
    val quality_height: Long? = null,
    val audio_languages: List<String> = emptyList(),
    val presentation: SourcePresentation? = null,
)

@Serializable
data class SourcePresentation(
    val provider_name: String? = null,
    val source_id: String? = null,
    val delivery_format: String? = null,
    val container: String? = null,
    val quality: String? = null,
    val video_codec: String? = null,
    val audio_codecs: List<String> = emptyList(),
    val size_bytes: Long? = null,
    val size_label: String? = null,
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

