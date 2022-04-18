// Copyright (c) 2022 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use nix::mount::MsFlags;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::snapshots::{MountPoint, SnapshotType, Snapshotter};

use std::fs::File;
use std::io::Write;
use std::io::Read;
 
//-----
use fs_extra;
use fs_extra::dir;
use fs_extra::error::*;
//-----

#[derive(Debug)]
pub struct OverLay {
    pub data_dir: PathBuf,
    pub index: AtomicUsize,
}

impl Snapshotter for OverLay {
    fn mount(&mut self, layer_path: &[&str], mount_path: &Path) -> Result<MountPoint> {
        let mnt_target = Path::new("/mnt_target");    
        fs::create_dir(mnt_target)?;

        if !mount_path.exists() {
            fs::create_dir_all(mount_path)?;
        }
        let fs_type = String::from("unionfs");
        let source  = Path::new(&fs_type);

        let options = format!(
            "lowerdir={},upperdir={},key={}",
            "./rootfs/lower",
            "./rootfs/upper",
            "c7-32-b3-ed-44-df-ec-7b-25-2d-9a-32-38-8d-58-61"
        );

        //let options = String::from("lowerdir=./rootfs/lower,upperdir=./rootfs/upper,key=c7-32-b3-ed-44-df-ec-7b-25-2d-9a-32-38-8d-58-61");
        let flags   = MsFlags::empty();

        println!("{:#?} {:#?} {:#?} {:#?} {:#?}", source, mount_path, fs_type, flags, options);
        nix::mount::mount(
            Some(source),
            mount_path,
            Some(fs_type.as_str()),
            flags,
            Some(options.as_str()),
        )?;
        println!("Mount done");

        //let create_file = mnt_target.join("foo.txt");
        //let mut file = File::create(create_file.as_path())?;
        //file.write_all(b"Hello, world!")?;
        //let mut file = File::open(create_file.as_path())?;
        //let mut contents = String::new();
        //file.read_to_string(&mut contents)?;
        //println!("{:#?}", contents);


        //------------------
        //let mut from_paths = Vec::new();
        //from_paths.push(layer_path[0]);
        //let options = dir::CopyOptions::new();
        //let result = fs_extra::copy_items(&from_paths, &mnt_target, &options).unwrap();
        
        //let mut from_paths = Vec::new();
        //let paths = fs::read_dir(mnt_target.to_str().unwrap()).unwrap();
        //for path in paths {
        //    println!("Name: {}", path.unwrap().path().display());
        //    from_paths.push(path.path());
        //}
        //let result = fs_extra::copy_items(&from_paths, &mnt_target, &options).unwrap();

        let options = dir::CopyOptions::new();
        for layer_dir in layer_path {
            let dirs = fs::read_dir(layer_dir).unwrap();
            let mut from_paths = Vec::new();
            for path in dirs {
                println!("Name: {}", path.as_ref().unwrap().path().display());
                from_paths.push(path.unwrap().path());
            }
            let result = fs_extra::copy_items(&from_paths, &mount_path, &options).unwrap();
        }
        //let root_dir = layer_path.join("layers");
        //println!("{:#?}", root_dir);
        //let dirs = fs::read_dir(root_dir.as_str().unwrap()).unwrap();
        //println!("dirs = {:#?}", dirs);

        //for path in dirs{
        //    let src_dir = root_dir.join(path);
        //    println!("Name: {}", src_dir.unwrap().path().display());
        //    let mut from_paths = Vec::new();
        //    from_paths.push(src_dir.as_path());
        //    let options = dir::CopyOptions::new();
        //    let result = copy_items(&from_paths, &mnt_target, &options).unwrap();
        //}
        //-----------------



        let paths = fs::read_dir(mount_path.to_str().unwrap()).unwrap();
        println!("paths = {:#?}",paths);
        for path in paths {
            println!("Name: {}", path.unwrap().path().display());
        }

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
