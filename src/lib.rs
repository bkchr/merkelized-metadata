use extrinsic_decoder::decode_extrinsic_and_collect_type_ids;
use frame_metadata::RuntimeMetadata;
use from_frame_metadata::FrameMetadataPrepared;
use merkle_tree::{MerkleTree, Proof};
use types::MetadataDigest;

mod extrinsic_decoder;
mod from_frame_metadata;
mod merkle_tree;
mod types;

/// Extra information that is required to generate the [`MetadataDigest`].
#[derive(Debug, Clone)]
pub struct ExtraInfo {
    pub spec_version: u32,
    pub spec_name: String,
    pub base58_prefix: u16,
    pub decimals: u8,
    pub token_symbol: String,
}

/// Generate the [`MetadataDigest`] using the given `extra_info`.
pub fn generate_metadata_digest(
    metadata: &RuntimeMetadata,
    extra_info: ExtraInfo,
) -> Result<MetadataDigest, String> {
    let prepared = FrameMetadataPrepared::prepare(metadata)?;

    let type_information = prepared.as_type_information();

    let tree_root = MerkleTree::new(type_information.types.into_iter()).root();

    Ok(MetadataDigest::V1 {
        types_tree_root: tree_root,
        extrinsic_metadata_hash: type_information.extrinsic_metadata.hash(),
        spec_version: extra_info.spec_version,
        spec_name: extra_info.spec_name,
        base58_prefix: extra_info.base58_prefix,
        decimals: extra_info.decimals,
        token_symbol: extra_info.token_symbol,
    })
}

/// Generate a proof for the given `extrinsic` using the given `metadata`.
///
/// `additonal_signed` can be passed as well to include the types required for decoding it in the proof as well.
pub fn generate_proof_for_extrinsic(
    extrinsic: &[u8],
    additional_signed: Option<&[u8]>,
    metadata: &RuntimeMetadata,
) -> Result<Proof, String> {
    let prepared = FrameMetadataPrepared::prepare(metadata)?;

    let accessed_types = decode_extrinsic_and_collect_type_ids(
        extrinsic,
        additional_signed,
        &prepared.as_type_information(),
    )?;

    MerkleTree::new(prepared.as_type_information().types).build_proof(accessed_types)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::frame_metadata::RuntimeMetadataPrefixed;
    use codec::Decode;
    use std::fs;

    const FIXTURES: &[(&str, &str)] = &[
        (
            "rococo_metadata_v15",
            "0x44e94d23868edef1b342ff8ee87832dce82676953b4a333736d108866106b91c",
        ),
        (
            "polkadot_metadata_v15",
            "0xacd252bffb7e6e2e3f5559d6d30557418481c87035382b57cd4d9bf1b5391341",
        ),
        (
            "kusama_metadata_v15",
            "0x5e38a03134f914e2ce185781a2e8359a1b5c8bbaffb421d75d1bbb9f86902ae7",
        ),
        (
            "acala_metadata_v15",
            "0xb5fe33037ef4fe0a243d0353b17770384c41380b4acfc304b7eae2d378a1d135",
        ),
        (
            "moonbeam_metadata_v15",
            "0x13f0aece8a3f0fb81e93ff8e2a0431a5a6a9570fb7f777739bbde65ab92a592f",
        ),
        (
            "hydradx_metadata_v15",
            "0x561190a68670c04aa8afe1208218a0f4a591b5e5a3ce4aa941ff32c6d3c09238",
        ),
    ];

    #[test]
    fn calculate_metadata_digest_works() {
        let extra_info = ExtraInfo {
            spec_version: 1,
            spec_name: "nice".into(),
            base58_prefix: 1,
            decimals: 1,
            token_symbol: "lol".into(),
        };

        for (fixture, expected_hash) in FIXTURES {
            println!("Processing: {fixture}");

            let metadata = String::from_utf8(
                fs::read(format!("{}/fixtures/{fixture}", env!("CARGO_MANIFEST_DIR"))).unwrap(),
            )
            .unwrap();

            let metadata = Option::<Vec<u8>>::decode(
                &mut &array_bytes::hex2bytes(metadata.strip_suffix("\n").unwrap()).unwrap()[..],
            )
            .unwrap()
            .unwrap();

            let metadata = RuntimeMetadataPrefixed::decode(&mut &metadata[..])
                .unwrap()
                .1;

            let digest = generate_metadata_digest(&metadata, extra_info.clone()).unwrap();
            assert_eq!(*expected_hash, array_bytes::bytes2hex("0x", &digest.hash()));

            let prepared = FrameMetadataPrepared::prepare(&metadata).unwrap();

            let type_information = prepared.as_type_information();
            type_information
                .types
                .values()
                .fold(None, |p, v| match p {
                    None => Some(v.clone()),
                    Some(p) => {
                        if p.type_id.0 < v.type_id.0
                            || p.type_def
                                .as_enumeration()
                                .and_then(|p| v.type_def.as_enumeration().map(|v| (p, v)))
                                .map_or(false, |(p, v)| p.index.0 < v.index.0)
                        {
                            Some(v.clone())
                        } else {
                            panic!("Invalid: {:?} < {:?}", p, v)
                        }
                    }
                })
                .unwrap();
        }
    }

    #[test]
    fn generate_proof() {
        // `Balances::transfer_keep_alive`
        let ext = "0x2d028400d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d01bce7c8f572d39cee240e3d50958f68a5c129e0ac0d4eb9222de70abdfa8c44382a78eded433782e6b614a97d8fd609a3f20162f3f3b3c16e7e8489b2bd4fa98c070000000403008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a4828";
        let additional_signed = "0x00b2590f001800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

        let metadata = String::from_utf8(
            fs::read(format!(
                "{}/fixtures/rococo_metadata_v15",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap(),
        )
        .unwrap();

        let metadata = Option::<Vec<u8>>::decode(
            &mut &array_bytes::hex2bytes(metadata.strip_suffix("\n").unwrap()).unwrap()[..],
        )
        .unwrap()
        .unwrap();

        let metadata = RuntimeMetadataPrefixed::decode(&mut &metadata[..])
            .unwrap()
            .1;

        let _proof = generate_proof_for_extrinsic(
            &array_bytes::hex2bytes(ext).unwrap(),
            Some(&array_bytes::hex2bytes(additional_signed).unwrap()),
            &metadata,
        )
        .unwrap();
    }
}
