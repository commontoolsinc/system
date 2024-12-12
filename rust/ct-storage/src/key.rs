use ranked_prolly_tree::Key as KeyTrait;

/// Key type in the [`CtStorage`].
pub type Key = Vec<u8>;

/// TODO Remove serde traits when no longer comparing
/// between bincoding
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MappedKey(Vec<u8>);
//pub struct MappedKey([u8; 96]);

impl MappedKey {
    pub fn new(entity: &str, ns: &str, attr: &str) -> Self {
        let mut key = [0u8; 96];
        for (i, item) in [entity, ns, attr].iter().enumerate() {
            let index = i * 32;
            let hash =
                <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(item.as_bytes())).to_vec();
            key[index..index + 32].copy_from_slice(&hash);
        }
        MappedKey(key.to_vec())
    }

    pub fn from_components(entity: &[u8], ns: &[u8], attr: &[u8]) -> Self {
        MappedKey([entity, ns, attr].concat())
    }

    pub fn entity(&self) -> &[u8] {
        &self.0[0..32]
    }

    pub fn ns(&self) -> &[u8] {
        &self.0[32..64]
    }

    pub fn attr(&self) -> &[u8] {
        &self.0[64..96]
    }
}

impl Ord for MappedKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.entity()
            .cmp(other.entity())
            .then(self.ns().cmp(other.ns()))
            .then(self.attr().cmp(other.attr()))
    }
}

impl PartialOrd for MappedKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MappedKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for MappedKey {}

impl AsRef<[u8]> for MappedKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl KeyTrait for MappedKey {}
