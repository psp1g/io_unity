use binrw::binrw;
use crate::serialized_file::version22::SerializedFileContent;

#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq)]
pub struct AssetFileHeader {
	pub _metadata_size: u32,
	pub _file_size: u32,
	pub version: u32,
	pub _data_offset: u32,
	pub endianness: u8,
	pub _unknown: [u8; 3],
	// Version >= 16
	//#[br(pad_before = 3)]
	pub metadata_size: u32,
	pub file_size: i64,
	pub data_offset: i64,
	pub _unknown2: [u8; 8],
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct AssetsFile {
	#[brw(big)]
	pub header: AssetFileHeader,
	#[brw(little)]
	pub content: SerializedFileContent,
}

