// Copyright (c) 2022 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use nix::mount::MsFlags;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::snapshots::{MountPoint, SnapshotType, Snapshotter};

#[derive(Debug)]
pub struct OverLay {
    pub data_dir: PathBuf,
    pub index: AtomicUsize,
}

impl Snapshotter for OverLay {
    fn mount(&mut self, layer_path: &[&str], mount_path: &Path) -> Result<MountPoint> {
        let mnt_target = Path::new("/mnt_target");    
        fs::create_dir(mnt_target)?;

        let fs_type = String::from("unionfs");
        let source = Path::new(&fs_type);
        let options = String::from("lowerdir=./rootfs/lower,upperdir=./rootfs/upper");
        let flags = MsFlags::empty();

        println!("{:#?} {:#?} {:#?} {:#?} {:#?}", source, mount_path, fs_type, flags, options);
        nix::mount::mount(
            Some(source),
            mnt_target,
            Some(fs_type.as_str()),
            flags,
            Some(options.as_str()),
        )?;
        println!("Mount done");

        Ok(MountPoint {
            r#type: fs_type,
            mount_path: mount_path.to_path_buf(),
            work_dir: self.data_dir.to_path_buf(),
        })

    }
//    fn mount(&mut self, layer_path: &[&str], mount_path: &Path) -> Result<MountPoint> {
//        let fs_type = SnapshotType::Overlay.to_string();
//        let overlay_lowerdir = layer_path.join(":");
//        let index = self.index.fetch_add(1, Ordering::SeqCst).to_string();
//        let work_dir = self.data_dir.join(&index);
//        let overlay_upperdir = work_dir.join("upperdir");
//        let overlay_workdir = work_dir.join("workdir");
//
//        if !self.data_dir.exists() {
//            fs::create_dir_all(&self.data_dir)?;
//        }
//        fs::create_dir_all(&overlay_upperdir)?;
//        fs::create_dir_all(&overlay_workdir)?;
//
//        if !mount_path.exists() {
//            fs::create_dir_all(mount_path)?;
//        }
//
//        let source = Path::new(&fs_type);
//        let flags = MsFlags::empty();
//        let options = format!(
//            "lowerdir={},upperdir={},workdir={}",
//            overlay_lowerdir,
//            overlay_upperdir.display(),
//            overlay_workdir.display()
//        );
//
//        println!("{:#?} {:#?} {:#?} {:#?} {:#?}", source, mount_path, fs_type.as_str(), flags, options.as_str());
//        nix::mount::mount(
//            Some(source),
//            mount_path,
//            Some(fs_type.as_str()),
//            flags,
//            Some(options.as_str()),
//        )
//        .map_err(|e| {
//            anyhow!(
//                "failed to mount {:?} to {:?}, with error: {}",
//                source,
//                mount_path,
//                e
//            )
//        })?;
//
//        Ok(MountPoint {
//            r#type: fs_type,
//            mount_path: mount_path.to_path_buf(),
//            work_dir,
//        })
//    }

    fn unmount(&self, mount_point: &MountPoint) -> Result<()> {
        nix::mount::umount(mount_point.mount_path.as_path())?;

        Ok(())
    }
}
