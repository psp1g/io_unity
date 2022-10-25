use std::{
    fmt,
    io::{Read, Seek, Write},
};

use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};

use crate::{def_unity_class, SerializedFileMetadata};

pub mod version13;
pub mod version14;

def_unity_class!(PPtr, PPtrObject);

pub trait PPtrObject: fmt::Debug {
    fn get_path_id(&self) -> i64;
}

impl BinRead for PPtr {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        if args.version.clone() as i32 >= 14 {
            return Ok(PPtr(Box::new(version14::PPtr::read_options(
                reader, options, args,
            )?)));
        }
        Ok(PPtr(Box::new(version13::PPtr::read_options(
            reader, options, args,
        )?)))
    }
}

impl BinWrite for PPtr {
    type Args = SerializedFileMetadata;

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        todo!();
        Ok(())
    }
}