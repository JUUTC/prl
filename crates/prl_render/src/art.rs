//! Presentation atlases keyed by content ID, independent of game rules.

use std::{collections::BTreeMap, fmt, path::PathBuf};

use bevy::{
    asset::RenderAssetUsages,
    image::{CompressedImageFormats, ImageSampler, ImageType},
    prelude::*,
};
use prl_content::{ContentId, ItemIcon};
use serde::Deserialize;

/// Manifest path relative to the content pack directory.
pub const PLACEHOLDER_MANIFEST: &str = "sprites/placeholder/gravebound_placeholder.ron";
/// Opt-in, pinned `UltiCa` crypt candidate; not final art approval.
pub const ULTICA_MANIFEST: &str = "sprites/ultica/ultica.ron";

/// Explicit presentation source; candidate failures never select another source.
#[derive(Resource, Debug, Clone)]
pub struct ArtSource {
    pack: PathBuf,
    candidate: bool,
    animated_player: bool,
}

impl ArtSource {
    /// Select the repository-owned procedural placeholder.
    #[must_use]
    pub fn placeholder(pack: PathBuf) -> Self {
        Self {
            pack,
            candidate: false,
            animated_player: false,
        }
    }

    /// Select the pinned `UltiCa` candidate for a bounded crypt proof.
    #[must_use]
    pub fn ultica(pack: PathBuf) -> Self {
        Self {
            pack,
            candidate: true,
            animated_player: false,
        }
    }

    /// Select `UltiCa` scenery with the earlier animated placeholder player.
    #[must_use]
    pub fn ultica_with_animated_player(pack: PathBuf) -> Self {
        Self {
            pack,
            candidate: true,
            animated_player: true,
        }
    }

    fn manifest(&self) -> &'static str {
        if self.candidate {
            ULTICA_MANIFEST
        } else {
            PLACEHOLDER_MANIFEST
        }
    }

    /// Validate image decoding and required candidate coverage before starting.
    ///
    /// # Errors
    /// Reports invalid/missing files, mismatched source status and missing mappings.
    pub fn validate_for(&self, state: &prl_core::CoreState) -> Result<(), ArtError> {
        let (index, _) = read(self)?;
        if self.animated_player {
            let (player, _) = read(&Self::placeholder(self.pack.clone()))?;
            for actor in state
                .content()
                .actors()
                .filter(|actor| actor.behavior == prl_core::ActorBehavior::Player)
            {
                let Some(sprites) = player.actors.get(&actor.id) else {
                    return Err(invalid(format!(
                        "animated player atlas missing {}",
                        actor.id
                    )));
                };
                if sprites.walk.is_none() || sprites.attack.is_none() {
                    return Err(invalid(format!(
                        "animated player {} needs walk and attack clips",
                        actor.id
                    )));
                }
            }
        }
        if self.candidate {
            for actor in state.content().actors() {
                if !index.actors.contains_key(&actor.id) {
                    return Err(invalid(format!(
                        "{}: missing actor {}",
                        self.manifest(),
                        actor.id
                    )));
                }
            }
            for tile in state.content().tiles() {
                let sprites = index.tiles.get(&tile.id).ok_or_else(|| {
                    invalid(format!("{}: missing tile {}", self.manifest(), tile.id))
                })?;
                if tile.door && sprites.open.is_none() {
                    return Err(invalid(format!(
                        "{}: missing open door {}",
                        self.manifest(),
                        tile.id
                    )));
                }
            }
            for marker in ["stairs_up", "stairs_down", "item_main_hand", "item_body"] {
                if !index.markers.contains_key(marker) {
                    return Err(invalid(format!(
                        "{}: missing marker {marker}",
                        self.manifest()
                    )));
                }
            }
            if let Some(approach) = state
                .content()
                .maps()
                .find(|map| map.id.to_string() == "prl:crypt_approach")
                && !index.map_tiles.get(&approach.id).is_some_and(|tiles| {
                    tiles.iter().any(|(tile, marker)| {
                        tile.to_string() == "prl:floor" && marker == "outdoor_grass"
                    })
                })
            {
                return Err(invalid("candidate is missing the outdoor grass mapping"));
            }
        }
        Ok(())
    }
}

type Rect = (u32, u32, u32, u32);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    status: String,
    #[expect(dead_code, reason = "Provenance for readers; not needed at runtime")]
    source: String,
    image: String,
    size: (u32, u32),
    #[serde(default = "unit_scale")]
    scale: f32,
    actors: BTreeMap<ContentId, ActorManifest>,
    tiles: BTreeMap<ContentId, TileManifest>,
    #[serde(default)]
    markers: BTreeMap<String, TileManifest>,
    #[serde(default)]
    map_tiles: BTreeMap<ContentId, BTreeMap<ContentId, String>>,
}

fn unit_scale() -> f32 {
    1.0
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActorManifest {
    #[serde(default)]
    offset_x: f32,
    offset_y: f32,
    idle: ClipManifest,
    walk: Option<ClipManifest>,
    attack: Option<ClipManifest>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClipManifest {
    frame_ms: u32,
    right: Vec<Rect>,
    left: Option<Vec<Rect>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TileManifest {
    rect: Rect,
    #[serde(default)]
    offset_x: f32,
    offset_y: f32,
    open: Option<Rect>,
}

/// Facing used to choose an independently authored sprite set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Facing {
    Left,
    #[default]
    Right,
}

/// Cosmetic action shown by an actor sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Action {
    #[default]
    Idle,
    Walk,
    Attack,
}

/// Atlas indices for one animation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Clip {
    frame_ms: u32,
    right: Vec<usize>,
    /// `None` means a single-facing study: the right set is used unmirrored.
    left: Option<Vec<usize>>,
}

impl Clip {
    fn frames(&self, facing: Facing) -> &[usize] {
        match (facing, &self.left) {
            (Facing::Left, Some(left)) => left,
            _ => &self.right,
        }
    }

    /// Duration of one full play-through in milliseconds.
    fn duration_ms(&self) -> u128 {
        u128::from(self.frame_ms) * self.right.len() as u128
    }

    /// Frame for a time offset; looping wraps, one-shots return `None` when done.
    fn frame_at(&self, facing: Facing, elapsed_ms: u128, looping: bool) -> Option<usize> {
        let frames = self.frames(facing);
        let step = usize::try_from(elapsed_ms / u128::from(self.frame_ms)).ok()?;
        if looping {
            frames.get(step % frames.len()).copied()
        } else {
            frames.get(step).copied()
        }
    }
}

/// Sprite data for one actor definition.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ActorSprites {
    pub(crate) offset: Vec2,
    idle: Clip,
    walk: Option<Clip>,
    attack: Option<Clip>,
}

impl ActorSprites {
    /// Fit a walk clip to the cosmetic tile traversal, then return to idle.
    pub(crate) fn walk_frame(
        &self,
        facing: Facing,
        elapsed_ms: u128,
        duration_ms: u128,
        now_ms: u128,
    ) -> usize {
        if duration_ms > 0
            && elapsed_ms < duration_ms
            && let Some(walk) = &self.walk
            && let Some(index) = walk.frame_at(
                facing,
                elapsed_ms.saturating_mul(walk.duration_ms()) / duration_ms,
                false,
            )
        {
            return index;
        }
        self.frame(facing, Action::Idle, 0, now_ms)
    }

    /// Atlas index for a cosmetic action started `since_ms` ago at `now_ms`.
    ///
    /// Walk/attack play once and then fall back to the looping idle clip.
    pub(crate) fn frame(
        &self,
        facing: Facing,
        action: Action,
        since_ms: u128,
        now_ms: u128,
    ) -> usize {
        let one_shot = match action {
            Action::Idle => None,
            Action::Walk => self.walk.as_ref(),
            Action::Attack => self.attack.as_ref(),
        };
        if let Some(index) = one_shot.and_then(|clip| clip.frame_at(facing, since_ms, false)) {
            return index;
        }
        let idle_time = one_shot.map_or(now_ms, |clip| since_ms.saturating_sub(clip.duration_ms()));
        self.idle
            .frame_at(facing, idle_time, true)
            .unwrap_or(self.idle.right[0])
    }
}

/// Sprite data for one tile definition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TileSprites {
    pub(crate) closed: usize,
    pub(crate) open: Option<usize>,
    pub(crate) offset: Vec2,
}

/// Validated atlas lookup, independent of Bevy assets.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AtlasIndex {
    pub(crate) image: String,
    pub(crate) size: UVec2,
    pub(crate) scale: f32,
    pub(crate) rects: Vec<URect>,
    pub(crate) actors: BTreeMap<ContentId, ActorSprites>,
    pub(crate) tiles: BTreeMap<ContentId, TileSprites>,
    pub(crate) markers: BTreeMap<String, TileSprites>,
    pub(crate) map_tiles: BTreeMap<ContentId, BTreeMap<ContentId, String>>,
}

/// Loaded presentation atlas used by the map renderer.
#[derive(Resource, Debug)]
pub(crate) struct AtlasArt {
    pub(crate) image: Handle<Image>,
    pub(crate) layout: Handle<TextureAtlasLayout>,
    pub(crate) index: AtlasIndex,
}

/// Read-only item art shared with sibling presentation crates by the binary.
///
/// Missing consumable entries are deliberate category/text placeholders, not
/// effect-dependent fallbacks. Handles never enter core state or snapshots.
#[derive(Resource, Debug)]
pub struct ItemIconAtlas {
    /// The already validated world atlas image.
    pub image: Handle<Image>,
    /// Native icon rectangles keyed only by public physical category.
    pub rects: BTreeMap<ItemIcon, URect>,
}

#[derive(Resource, Debug)]
pub(crate) struct PlayerArt(pub(crate) AtlasArt);

impl AtlasArt {
    pub(crate) fn sprite(&self, index: usize) -> Sprite {
        let mut sprite = Sprite::from_atlas_image(
            self.image.clone(),
            TextureAtlas {
                layout: self.layout.clone(),
                index,
            },
        );
        if let Some(rect) = self.index.rects.get(index) {
            sprite.custom_size = Some(rect.size().as_vec2() * self.index.scale);
        }
        sprite
    }
}

/// Why a presentation atlas could not be read or validated.
#[derive(Debug)]
pub struct ArtError(String);

impl fmt::Display for ArtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ArtError {}

fn invalid(message: impl Into<String>) -> ArtError {
    ArtError(message.into())
}

fn validate_manifest(manifest: &Manifest) -> Result<(), ArtError> {
    if !matches!(manifest.schema_version, 1 | 2) {
        return Err(invalid(format!(
            "unsupported schema_version {} (expected 1 or 2)",
            manifest.schema_version
        )));
    }
    if manifest.status != "placeholder"
        && !(manifest.schema_version == 2 && manifest.status == "candidate")
    {
        return Err(invalid("expected placeholder or schema-2 candidate status"));
    }
    if !manifest.scale.is_finite() || !(0.25..=8.0).contains(&manifest.scale) {
        return Err(invalid(
            "sprite scale must be finite and between 0.25 and 8",
        ));
    }
    let image_path = std::path::Path::new(&manifest.image);
    if image_path.components().count() != 1
        || !matches!(
            image_path.components().next(),
            Some(std::path::Component::Normal(_))
        )
        || manifest.image.contains(['\\', ':'])
    {
        return Err(invalid("image must be an adjacent file name"));
    }
    if manifest.size.0 == 0
        || manifest.size.1 == 0
        || manifest.size.0 > 8192
        || manifest.size.1 > 8192
    {
        return Err(invalid("atlas dimensions must be in 1..=8192"));
    }
    Ok(())
}

/// Parse and validate a manifest; every rectangle must fit the declared atlas.
pub(crate) fn parse_manifest(text: &str) -> Result<AtlasIndex, ArtError> {
    let manifest: Manifest =
        ron::from_str(text).map_err(|error| invalid(format!("invalid manifest: {error}")))?;
    validate_manifest(&manifest)?;
    let size = UVec2::new(manifest.size.0, manifest.size.1);
    let mut rects = Vec::new();
    let mut add = |rect: Rect| -> Result<usize, ArtError> {
        let (x, y, w, h) = rect;
        let fits = w > 0
            && h > 0
            && x.checked_add(w).is_some_and(|right| right <= size.x)
            && y.checked_add(h).is_some_and(|bottom| bottom <= size.y);
        if !fits {
            return Err(invalid(format!(
                "rectangle {rect:?} lies outside the atlas"
            )));
        }
        rects.push(URect::new(x, y, x + w, y + h));
        Ok(rects.len() - 1)
    };
    let mut clip = |clip: ClipManifest| -> Result<Clip, ArtError> {
        if clip.frame_ms == 0 || clip.right.is_empty() {
            return Err(invalid("clips need frames and a positive frame_ms"));
        }
        if clip
            .left
            .as_ref()
            .is_some_and(|left| left.len() != clip.right.len())
        {
            return Err(invalid("left and right facings need equal frame counts"));
        }
        let right = clip
            .right
            .into_iter()
            .map(&mut add)
            .collect::<Result<_, _>>()?;
        let left = clip
            .left
            .map(|left| left.into_iter().map(&mut add).collect::<Result<_, _>>())
            .transpose()?;
        Ok(Clip {
            frame_ms: clip.frame_ms,
            right,
            left,
        })
    };
    let mut actors = BTreeMap::new();
    for (id, actor) in manifest.actors {
        let offset = checked_offset(actor.offset_x, actor.offset_y)?;
        let sprites = ActorSprites {
            offset,
            idle: clip(actor.idle)?,
            walk: actor.walk.map(&mut clip).transpose()?,
            attack: actor.attack.map(&mut clip).transpose()?,
        };
        actors.insert(id, sprites);
    }
    let mut tile = |tile: TileManifest| -> Result<TileSprites, ArtError> {
        Ok(TileSprites {
            closed: add(tile.rect)?,
            open: tile.open.map(&mut add).transpose()?,
            offset: checked_offset(tile.offset_x, tile.offset_y)?,
        })
    };
    let tiles = manifest
        .tiles
        .into_iter()
        .map(|(id, entry)| Ok((id, tile(entry)?)))
        .collect::<Result<_, ArtError>>()?;
    let markers: BTreeMap<String, TileSprites> = manifest
        .markers
        .into_iter()
        .map(|(id, entry)| Ok((id, tile(entry)?)))
        .collect::<Result<_, ArtError>>()?;
    for mappings in manifest.map_tiles.values() {
        for marker in mappings.values() {
            if !markers.contains_key(marker) {
                return Err(invalid(format!("unknown map tile marker {marker}")));
            }
        }
    }
    Ok(AtlasIndex {
        image: manifest.image,
        size,
        scale: manifest.scale,
        rects,
        actors,
        tiles,
        markers,
        map_tiles: manifest.map_tiles,
    })
}

fn checked_offset(x: f32, y: f32) -> Result<Vec2, ArtError> {
    let offset = Vec2::new(x, y);
    if !offset.is_finite() || offset.abs().max_element() > 8192.0 {
        return Err(invalid("sprite offsets must be finite and bounded by 8192"));
    }
    Ok(offset)
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, load.run_if(resource_exists::<ArtSource>));
}

fn load(
    mut commands: Commands<'_, '_>,
    source: Res<'_, ArtSource>,
    images: Option<ResMut<'_, Assets<Image>>>,
    layouts: Option<ResMut<'_, Assets<TextureAtlasLayout>>>,
    mut exit: MessageWriter<'_, AppExit>,
) {
    let (Some(mut images), Some(mut layouts)) = (images, layouts) else {
        error!("Image assets are unavailable");
        if source.candidate {
            exit.write(AppExit::error());
        }
        return;
    };
    match read(&source) {
        Ok((index, image)) => {
            let mut layout = TextureAtlasLayout::new_empty(index.size);
            for rect in &index.rects {
                layout.add_texture(*rect);
            }
            info!(
                "Loaded presentation atlas {} ({} actors, {} tiles)",
                index.image,
                index.actors.len(),
                index.tiles.len()
            );
            let image = images.add(image);
            let rects = [
                ItemIcon::MainHand,
                ItemIcon::Body,
                ItemIcon::Potion,
                ItemIcon::Scroll,
            ]
            .into_iter()
            .filter_map(|icon| {
                let marker = index.markers.get(icon.marker())?;
                let rect = index.rects.get(marker.closed)?;
                Some((icon, *rect))
            })
            .collect();
            commands.insert_resource(ItemIconAtlas {
                image: image.clone(),
                rects,
            });
            commands.insert_resource(AtlasArt {
                image,
                layout: layouts.add(layout),
                index,
            });
            if source.animated_player {
                match read(&ArtSource::placeholder(source.pack.clone())) {
                    Ok((index, image)) => {
                        let mut layout = TextureAtlasLayout::new_empty(index.size);
                        for rect in &index.rects {
                            layout.add_texture(*rect);
                        }
                        commands.insert_resource(PlayerArt(AtlasArt {
                            image: images.add(image),
                            layout: layouts.add(layout),
                            index,
                        }));
                    }
                    Err(error) => {
                        error!("Animated player atlas unavailable: {error}");
                        exit.write(AppExit::error());
                    }
                }
            }
        }
        Err(error) => {
            error!(
                "Atlas unavailable ({}): {error}",
                source.pack.join(source.manifest()).display()
            );
            if source.candidate {
                exit.write(AppExit::error());
            }
        }
    }
}

pub(crate) fn read(source: &ArtSource) -> Result<(AtlasIndex, Image), ArtError> {
    let manifest_path = source.pack.join(source.manifest());
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|error| invalid(format!("cannot read {}: {error}", manifest_path.display())))?;
    let manifest: Manifest = ron::from_str(&text)
        .map_err(|error| invalid(format!("{}: {error}", manifest_path.display())))?;
    if source.candidate != (manifest.status == "candidate") {
        return Err(invalid(format!(
            "{}: status does not match selected source",
            manifest_path.display()
        )));
    }
    let index = parse_manifest(&text)
        .map_err(|error| invalid(format!("{}: {error}", manifest_path.display())))?;
    let image_path = manifest_path.with_file_name(&index.image);
    let bytes = std::fs::read(&image_path)
        .map_err(|error| invalid(format!("cannot read {}: {error}", image_path.display())))?;
    let image = Image::from_buffer(
        &bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::nearest(),
        RenderAssetUsages::default(),
    )
    .map_err(|error| invalid(format!("cannot decode {}: {error}", image_path.display())))?;
    if image.size() != index.size {
        return Err(invalid(format!(
            "image is {} but the manifest declares {}",
            image.size(),
            index.size
        )));
    }
    Ok((index, image))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/prl")
    }

    fn id(name: &str) -> Result<ContentId, Box<dyn std::error::Error>> {
        Ok(ContentId::new("prl", name)?)
    }

    #[test]
    fn shipped_atlas_covers_base_pack_definitions() -> Result<(), Box<dyn std::error::Error>> {
        let (index, _) = read(&ArtSource::placeholder(pack()))?;
        let content = prl_content::ContentPack::load_from_path(pack().join("base.ron"))?;
        for actor in content.actors() {
            assert!(
                index.actors.contains_key(&actor.id),
                "no sprite for {}",
                actor.id
            );
        }
        for tile in content.tiles() {
            assert!(
                index.tiles.contains_key(&tile.id),
                "no sprite for {}",
                tile.id
            );
        }
        // Every base-pack actor has independent facings for idle, walk and attack.
        for actor in content.actors() {
            let art = &index.actors[&actor.id];
            for clip in [Some(&art.idle), art.walk.as_ref(), art.attack.as_ref()] {
                let clip = clip.ok_or_else(|| format!("{} clip missing", actor.id))?;
                assert!(
                    clip.left.as_ref().is_some_and(|left| left != &clip.right),
                    "{} needs an independent left facing",
                    actor.id
                );
            }
        }
        assert!(index.tiles[&id("door")?].open.is_some());
        Ok(())
    }

    #[test]
    fn manifests_reject_out_of_bounds_and_non_placeholder_data() {
        let base = r#"(schema_version: 1, status: "placeholder", source: "", image: "a.png",
            size: (64, 64), actors: {}, tiles: {"prl:floor": (rect: RECT, offset_y: 0, open: None)})"#;
        assert!(parse_manifest(&base.replace("RECT", "(0, 0, 64, 64)")).is_ok());
        assert!(parse_manifest(&base.replace("RECT", "(1, 0, 64, 64)")).is_err());
        assert!(parse_manifest(&base.replace("RECT", "(0, 0, 0, 64)")).is_err());
        assert!(
            parse_manifest(
                &base
                    .replace("RECT", "(0, 0, 64, 64)")
                    .replace("\"placeholder\"", "\"final\"")
            )
            .is_err()
        );
    }

    #[test]
    fn candidate_covers_the_crypt_with_static_unmirrored_scaled_sprites()
    -> Result<(), Box<dyn std::error::Error>> {
        let source = ArtSource::ultica(pack());
        source.validate_for(&prl_core::CoreState::demo()?)?;
        let (index, _) = read(&source)?;
        assert_eq!(index.scale, 2.0);
        assert_eq!(index.size, UVec2::new(442, 66));
        assert_eq!(index.markers.len(), 5);
        for icon in [ItemIcon::MainHand, ItemIcon::Body] {
            assert_eq!(
                index.rects[index.markers[icon.marker()].closed].size(),
                UVec2::splat(32)
            );
        }
        for icon in [ItemIcon::Potion, ItemIcon::Scroll] {
            assert!(!index.markers.contains_key(icon.marker()));
        }
        assert_eq!(
            index.map_tiles[&id("crypt_approach")?][&id("floor")?],
            "outdoor_grass"
        );
        ArtSource::ultica_with_animated_player(pack())
            .validate_for(&prl_core::CoreState::demo()?)?;
        assert_ne!(
            index.tiles[&id("door")?].open,
            Some(index.tiles[&id("door")?].closed)
        );
        for actor in index.actors.values() {
            assert_eq!(actor.offset, Vec2::new(0.0, 32.0));
            assert!(actor.idle.left.is_none());
            assert!(actor.walk.is_none());
            assert!(actor.attack.is_none());
            let frame = actor.frame(Facing::Right, Action::Idle, 0, 0);
            assert_eq!(frame, actor.frame(Facing::Left, Action::Attack, 500, 500));
            assert_eq!(
                index.rects[frame].size().as_vec2() * index.scale,
                Vec2::new(64.0, 128.0)
            );
        }
        Ok(())
    }

    #[test]
    fn invalid_candidate_files_or_coverage_never_load_the_placeholder()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let source = ArtSource::ultica(directory.path().to_path_buf());
        let target = directory.path().join(ULTICA_MANIFEST);
        std::fs::create_dir_all(target.parent().ok_or("missing parent")?)?;
        let state = prl_core::CoreState::demo()?;
        assert!(source.validate_for(&state).is_err());
        let text = std::fs::read_to_string(pack().join(ULTICA_MANIFEST))?;
        std::fs::write(&target, &text)?;
        assert!(
            source.validate_for(&state).is_err(),
            "missing image rejects"
        );
        std::fs::copy(
            pack().join("sprites/ultica/ultica.png"),
            target.with_file_name("ultica.png"),
        )?;
        source.validate_for(&state)?;
        for changed in [
            text.replace("\"prl:skeleton\"", "\"prl:missing_actor\""),
            text.replace("\"prl:floor\"", "\"prl:missing_tile\""),
            text.replace("\"stairs_up\"", "\"missing_marker\""),
            text.replace("\"item_main_hand\"", "\"missing_item\""),
            text.replace("\"item_body\"", "\"missing_item\""),
            text.replace("status: \"candidate\"", "status: \"placeholder\""),
            text.replace("size: (442, 66)", "size: (443, 66)"),
        ] {
            std::fs::write(&target, changed)?;
            assert!(source.validate_for(&state).is_err());
        }
        std::fs::write(&target, text)?;
        std::fs::write(target.with_file_name("ultica.png"), b"broken png")?;
        assert!(source.validate_for(&state).is_err());
        Ok(())
    }

    #[test]
    fn manifests_reject_unsafe_paths_scales_and_nonfinite_offsets() {
        let text = include_str!("../../../assets/packs/prl/sprites/ultica/ultica.ron");
        for changed in [
            text.replace("scale: 2.0", "scale: 0.0"),
            text.replace("scale: 2.0", "scale: NaN"),
            text.replace("offset_y: 32.0", "offset_y: inf"),
            text.replace("\"ultica.png\"", "\"../ultica.png\""),
            text.replace("\"ultica.png\"", "\"C:ultica.png\""),
            text.replace("size: (442, 66)", "size: (9000, 66)"),
        ] {
            assert!(parse_manifest(&changed).is_err(), "{changed}");
        }
    }

    #[test]
    fn one_shot_actions_return_to_idle_and_single_facing_is_not_mirrored() {
        let clip = |start: usize| Clip {
            frame_ms: 100,
            right: vec![start, start + 1],
            left: Some(vec![start + 2, start + 3]),
        };
        let sprites = ActorSprites {
            offset: Vec2::ZERO,
            idle: clip(0),
            walk: Some(clip(10)),
            attack: None,
        };
        assert_eq!(sprites.frame(Facing::Right, Action::Walk, 0, 5_000), 10);
        assert_eq!(sprites.walk_frame(Facing::Right, 0, 150, 5_000), 10);
        assert_eq!(sprites.walk_frame(Facing::Right, 75, 150, 5_000), 11);
        assert_eq!(sprites.walk_frame(Facing::Right, 150, 150, 5_000), 0);
        assert_eq!(sprites.walk_frame(Facing::Right, 0, 0, 5_000), 0);
        assert_eq!(sprites.frame(Facing::Left, Action::Walk, 150, 5_000), 13);
        // Walk lasts 200ms, then idle resumes from its first frame.
        assert_eq!(sprites.frame(Facing::Left, Action::Walk, 200, 5_000), 2);
        // Missing clips fall back to the looping idle.
        assert_eq!(sprites.frame(Facing::Right, Action::Attack, 0, 100), 1);
        let single = Clip {
            left: None,
            ..clip(20)
        };
        assert_eq!(single.frames(Facing::Left), &[20, 21]);
    }
}
