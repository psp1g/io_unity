pub mod type_tree;

use crate::def_unity_class;
use crate::error::{Error, ReadResult};
use crate::serialized_file::SerializedFile;
use crate::type_tree::TypeTreeObject;
use crate::unity_asset_view::UnityAssetViewer;

use std::path::PathBuf;

use super::SerializedFileRef;

def_unity_class!(PPtr);

pub trait PPtrObject: SerializedFileRef {
    fn get_path_id(&self) -> ReadResult<i64>;
    fn get_file_id(&self) -> ReadResult<i64>;

    fn get_serialized_file<'a>(
        &self,
        self_serialized_file: &'a SerializedFile,
        viewer: Option<&'a UnityAssetViewer>,
    ) -> ReadResult<&'a SerializedFile> {
        let file_id = self.get_file_id()?;

        if file_id == 0 {
            return Ok(self_serialized_file);
        }

        if let Some(viewer) = viewer {
            let externals = self_serialized_file.get_externals();

            if file_id > 0 {
                if let Some(external) = externals.get(file_id as usize - 1) {
                    if let Some(file_name) = PathBuf::from(&external.path.to_string())
                        .file_name()
                        .map(|f| f.to_string_lossy().into_owned())
                    {
                        if let Some(serialized_file) =
                            viewer.get_serialized_file_by_path(&file_name)
                        {
                            return Ok(serialized_file);
                        }
                    }
                }
            }
        }
        Err(Error::ExternalSerializedFileNotFound)
    }

    fn get_type_tree_object(
        &self,
        self_serialized_file: &SerializedFile,
        viewer: Option<&UnityAssetViewer>,
    ) -> Result<Option<TypeTreeObject>, Error> {
        let path_id = self.get_path_id()?;
        let serialized_file = self.get_serialized_file(self_serialized_file, viewer)?;
        serialized_file.get_tt_object_by_path_id(path_id)
    }

    fn get_type_tree_object_in_view(
        &self,
        viewer: &UnityAssetViewer,
    ) -> Result<Option<TypeTreeObject>, Error> {
        let self_serialized_file = viewer
            .serialized_file_map
            .get(&self.get_serialized_file_id())
            .ok_or(Error::SerializedFileNotFound)?;
        self.get_type_tree_object(self_serialized_file, Some(viewer))
    }
}
