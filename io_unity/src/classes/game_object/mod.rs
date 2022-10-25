pub mod version_5_5_0;

use std::{
    fmt,
    io::{Read, Seek, SeekFrom, Write},
};

use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};

use crate::{
    def_unity_class,
    until::{binrw_parser::AlignedString, UnityVersion},
    SerializedFileMetadata,
};

def_unity_class!(GameObject, GameObjectObject);

pub trait GameObjectObject: fmt::Debug {
    fn get_name(&self) -> &AlignedString;
}

impl BinRead for GameObject {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        if args.unity_version >= UnityVersion::new(vec![5, 5], None) {
            return Ok(GameObject(Box::new(
                version_5_5_0::GameObject::read_options(reader, options, args)?,
            )));
        }
        Err(binrw::Error::NoVariantMatch {
            pos: reader.seek(SeekFrom::Current(0)).unwrap(),
        })
    }
}

impl BinWrite for GameObject {
    type Args = SerializedFileMetadata;

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Ok(())
    }
}