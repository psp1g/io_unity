use std::borrow::Cow;

use super::version17::{read_type_tree_string, FileIdentifier, Object, ScriptType};
use super::{BuildTarget, SerializedFileCommonHeader};
use super::{Serialized, SerializedFileFormatVersion};
use crate::type_tree::{reader::TypeTreeObjectBinReadClassArgs, TypeField};
use crate::until::{binrw_parser::*, Endian};
use binrw::io::Cursor;
use binrw::{binrw, NullString};
use std::fmt;
use std::io::prelude::*;
use std::sync::Arc;

#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq)]
pub struct SerializedFile {
    header: SerializedFileCommonHeader,
    endianess: Endian,
    reserved: [u8; 3],
    #[br(is_little = endianess == Endian::Little)]
    content: SerializedFileContent,
}

impl Serialized for SerializedFile {
    fn get_serialized_file_version(&self) -> &SerializedFileFormatVersion {
        &self.header.version
    }

    fn get_data_offset(&self) -> u64 {
        self.header.data_offset as u64
    }

    fn get_endianess(&self) -> &Endian {
        &self.endianess
    }

    fn get_objects_metadata(&self) -> Vec<super::Object> {
        self.content
            .objects
            .iter()
            .map(|obj| super::Object {
                path_id: obj.path_id,
                byte_start: obj.byte_start as u64,
                byte_size: obj.byte_size,
                class: self
                    .content
                    .types
                    .get(obj.type_id as usize)
                    .map(|t| t.class_id)
                    .unwrap_or(0),
                type_id: obj.type_id as usize,
            })
            .collect()
    }

    fn get_unity_version(&self) -> String {
        self.content.unity_version.to_string()
    }

    fn get_target_platform(&self) -> &BuildTarget {
        &self.content.target_platform
    }

    fn get_enable_type_tree(&self) -> bool {
        *self.content.enable_type_tree
    }

    fn get_type_object_args_by_type_id(
        &self,
        type_id: usize,
    ) -> Option<TypeTreeObjectBinReadClassArgs> {
        let stypetree = &self.content.types.get(type_id)?;
        let type_tree = stypetree.type_tree.as_ref()?;
        let mut type_fields = Vec::new();
        let mut string_reader = Cursor::new(&type_tree.string_buffer);

        for tp in &type_tree.type_tree_node_blobs {
            type_fields.push(Arc::new(Box::new(TypeTreeNode {
                name: tp.get_name_str(&mut string_reader),
                type_name: tp.get_type_str(&mut string_reader),
                node: tp.clone(),
            }) as Box<dyn TypeField + Send + Sync>))
        }

        Some(TypeTreeObjectBinReadClassArgs::new(
            stypetree.class_id,
            type_fields,
        ))
    }
    fn get_externals(&self) -> Cow<Vec<FileIdentifier>> {
        return Cow::Borrowed(&self.content.externals);
    }
}

#[binrw]
#[derive(Debug, PartialEq)]
struct SerializedFileContent {
    unity_version: NullString,
    target_platform: BuildTarget,
    enable_type_tree: U8Bool,
    type_count: u32,
    #[br(args { count: type_count as usize, inner: SerializedTypeBinReadArgs { enable_type_tree:*enable_type_tree } })]
    types: Vec<SerializedType>,
    object_count: i32,
    #[br(count = object_count)]
    objects: Vec<Object>,
    script_count: i32,
    #[br(count = script_count)]
    script_types: Vec<ScriptType>,
    externals_count: i32,
    #[br(count = externals_count)]
    externals: Vec<FileIdentifier>,
    user_information: NullString,
}

#[binrw]
#[br(import { enable_type_tree: bool})]
#[derive(Debug, PartialEq)]
pub struct SerializedType {
    pub class_id: i32,
    pub is_stripped_type: U8Bool,
    pub script_type_index: i16,
    #[br(if(class_id == 114))]
    pub script_id: Option<[u8; 16]>,
    pub old_type_hash: [u8; 16],
    #[br(if(enable_type_tree))]
    pub type_tree: Option<TypeTree>,
}

#[binrw]
#[derive(Clone, PartialEq)]
pub struct TypeTree {
    number_of_nodes: i32,
    string_buffer_size: i32,
    #[br(count = number_of_nodes)]
    pub type_tree_node_blobs: Vec<TypeTreeNodeBlob>,
    #[br(count = string_buffer_size)]
    pub string_buffer: Vec<u8>,
}

impl fmt::Debug for TypeTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string_reader = Cursor::new(&self.string_buffer);

        write!(f, "TypeTree [")?;
        if f.alternate() {
            writeln!(f)?;
        }
        for node in &self.type_tree_node_blobs {
            write!(
                f,
                "{:?} -> {{ type: {}, name: {} }},",
                node,
                node.get_type_str(&mut string_reader),
                node.get_name_str(&mut string_reader)
            )?;
            if f.alternate() {
                writeln!(f)?;
            }
        }
        write!(f, "]")
    }
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
pub struct TypeTreeNodeBlob {
    version: u16,
    level: u8,
    type_flags: u8,
    type_str_offset: u32,
    name_str_offset: u32,
    byte_size: i32,
    index: i32,
    meta_flag: i32,
    ref_type_hash: u64,
}

impl TypeTreeNodeBlob {
    pub fn get_type_str<R: Read + Seek>(&self, reader: &mut R) -> String {
        read_type_tree_string(self.type_str_offset, reader)
    }

    pub fn get_name_str<R: Read + Seek>(&self, reader: &mut R) -> String {
        read_type_tree_string(self.name_str_offset, reader)
    }
}

#[derive(Debug, PartialEq)]
pub struct TypeTreeNode {
    pub name: String,
    pub type_name: String,
    pub node: TypeTreeNodeBlob,
}

impl TypeField for TypeTreeNode {
    fn get_version(&self) -> u16 {
        self.node.version
    }

    fn get_level(&self) -> u8 {
        self.node.level
    }
    //0x01 : IsArray
    //0x02 : IsRef
    //0x04 : IsRegistry
    //0x08 : IsArrayOfRefs
    fn is_array(&self) -> bool {
        self.node.type_flags & 1 > 0
    }

    fn get_byte_size(&self) -> i32 {
        self.node.byte_size
    }

    fn get_index(&self) -> i32 {
        self.node.index
    }

    //0x0001 : is invisible(?), set for m_FileID and m_PathID; ignored if no parent field exists or the type is neither ColorRGBA, PPtr nor string
    //0x0100 : ? is bool
    //0x1000 : ?
    //0x4000 : align bytes
    //0x8000 : any child has the align bytes flag
    //=> if flags & 0xC000 and size != 0xFFFFFFFF, the size field matches the total length of this field plus its children.
    //0x400000 : ?
    //0x800000 : ? is non-primitive type
    //0x02000000 : ? is UInt16 (called char)
    //0x08000000 : has fixed buffer size? related to Array (i.e. this field or its only child or its father is an array), should be set for vector, Array and the size and data fields.
    fn get_meta_flag(&self) -> i32 {
        self.node.meta_flag
    }

    fn is_align(&self) -> bool {
        self.node.meta_flag & 0x4000 > 0
    }

    fn get_ref_type_hash(&self) -> Option<u64> {
        Some(self.node.ref_type_hash)
    }

    fn get_type(&self) -> &String {
        &self.type_name
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}
