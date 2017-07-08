// Copyright 2017 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Main interface for the cuckoo_miner plugin manager, which queries
//! all available plugins in a particular directory and returns their
//! descriptions, parameters, and capabilities

use std::{fmt,cmp};
use std::env;
use std::path::{Path,PathBuf};
use std::io;
use std::path::MAIN_SEPARATOR;

use regex::Regex;
use glob::glob;

use cuckoo_sys::{call_cuckoo_description, load_cuckoo_lib};
use error::CuckooMinerError;


// OS-specific library extensions

#[cfg(target_os = "linux")]
static DLL_SUFFIX: &str=".so";
#[cfg(target_os = "macos")]
static DLL_SUFFIX: &str=".dylib";

// Helper function to get the absolute path from a relative path

fn abspath<P: AsRef<Path> + ?Sized>(relpath: &P) -> String { 
    let result=env::current_dir().map(|p| p.join(relpath.as_ref()));
    let full_path = result.unwrap();
    String::from(full_path.to_str().unwrap())
}

#[derive(Debug, Clone)]
pub struct CuckooPluginCapabilities {
    // The plugin's descriptive name
    // As reported by the plugin
    pub name: String,

    // The plugin's reported description
    pub description: String,

    // The full path to the plugin
    pub full_path: String,

    // The plugin's file name
    pub file_name: String,
}

impl Default for CuckooPluginCapabilities {
	fn default() -> CuckooPluginCapabilities {
		CuckooPluginCapabilities{
            name: String::from(""),
            description: String::from(""),
			full_path: String::from(""),
            file_name: String::from(""),
		}
	}
}

impl fmt::Display for CuckooPluginCapabilities{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Name: {}\nDescription:{}\nPath:{}\n", self.name, self.description, self.full_path)
    }
}

pub struct CuckooPluginManager {
    // The directory in which to look for plugins
    plugin_dir: String,

    //
    current_plugin_caps: Option<Vec<CuckooPluginCapabilities>>,
}

impl Default for CuckooPluginManager {
	fn default() -> CuckooPluginManager {
		CuckooPluginManager {
            plugin_dir: String::from("target/debug"),
            current_plugin_caps: None,
		}
	}
}

impl CuckooPluginManager {

    pub fn new()->Result<CuckooPluginManager, CuckooMinerError>{
        Ok(CuckooPluginManager::default())
    }

    pub fn load_plugin_dir (&mut self, plugin_dir:String) 
        -> Result<(), CuckooMinerError> {
        self.plugin_dir = plugin_dir.clone();
        let caps=self.load_all_plugin_caps(&plugin_dir)?;
        self.current_plugin_caps=Some(caps);
        Ok(())
    }

    pub fn get_available_plugins(&mut self, filter:&str) -> 
        Result<Vec<CuckooPluginCapabilities>, CuckooMinerError>{
            if filter.len()==0 {
                return Ok(self.current_plugin_caps.as_mut().unwrap().clone());
            } else {
                let result = self.current_plugin_caps.as_mut().unwrap().clone().into_iter().filter(
                    |ref i| {
                        let re = Regex::new(&format!(r"{}",filter)).unwrap();
                        let caps = re.captures(&i.full_path);
                        match caps {
                            Some(e) => return true,
                            None => return false,
                        }
                    }).collect::<Vec<_>>();
                if (result.len()==0){
                    return Err(CuckooMinerError::NoPluginsFoundError(format!("For given filter: {}", filter)));
                }
                return Ok(result);
            }
    }

    // Fills out and Returns a CuckooPluginCapabilities structure parsed from a
    // call to call_cuckoo_description in the currently loaded plugin

    fn load_plugin_caps(&mut self, full_path:String) 
        -> Result<CuckooPluginCapabilities, CuckooMinerError> {
            //println!("{:?}",path.display() );
            let mut caps=CuckooPluginCapabilities::default();
            load_cuckoo_lib(&full_path)?;
            let mut name_bytes:[u8;256]=[0;256];
            let mut description_bytes:[u8;256]=[0;256];
            let mut name_len=name_bytes.len() as u32;
            let mut desc_len=description_bytes.len() as u32;
            call_cuckoo_description(&mut name_bytes, &mut name_len, 
                                    &mut description_bytes, &mut desc_len);
            
            let mut name_vec:Vec<u8> = Vec::new();
            for i in 0..name_len {
                name_vec.push(name_bytes[i as usize].clone());
            }
            
            let mut desc_vec:Vec<u8> = Vec::new();
            for i in 0..desc_len {
                desc_vec.push(description_bytes[i as usize].clone());
            }
            
            caps.name=String::from_utf8(name_vec)?;
            caps.description=String::from_utf8(desc_vec)?;
            caps.full_path=full_path.clone();
            caps.file_name=String::from("");

            return Ok(caps);
    }

    /// #Description 
    ///
    /// Loads all available plugins in the plugin directory one by one,
    /// calls their cuckoo_description functions, and returns a vector
    /// of /// [CuckooPluginCapabilities](../../config/types/struct.CuckooPluginCapabilities.html) 
    /// representing the currently
    /// installed plugins on the system. This will parse any dll
    /// with the name 'cuckoo' in it, with suffix depending on the host os 
    /// (.so on unix, .dylib on mac, .dll on windows(not implemented as of yet))
    ///
    /// #Arguments
    ///
    /// * `plugin_dir` (IN) The path to the prefered plugin directory. This can
    /// be either relative to the current directory or a full path. This will
    /// be resolved to a full path before calling each plugin.
    ///
    /// #Returns
    ///
    /// If successful, a Result containing a vector of 
    /// [CuckooPluginCapabilities](../../config/types/struct.CuckooPluginCapabilities.html) , 
    /// one for each plugin successfully read from the plugin directory.
    /// If there is an error loading plugins from the given directory,
    /// a [CuckooMinerError](../../config/types/enum.CuckooMinerError.html) 
    /// will be returned outlining more specific details.
    ///
    /// #Example
    /// 
    /// ```
    /// let caps=get_available_plugins("./path/to/plugin/dir")?;
    /// //Print all available plugins
    /// for c in &caps {
    ///     println!("Found plugin: [{}]", c);
    /// }
    /// ```
    ///

    fn load_all_plugin_caps(&mut self, plugin_dir: &str) 
            -> Result<Vec<CuckooPluginCapabilities>,CuckooMinerError>{
        let lib_full_path = abspath(Path::new(&plugin_dir));
        let glob_search_path = format!("{}/*cuckoo*{}", lib_full_path, DLL_SUFFIX);

        let mut result_vec:Vec<CuckooPluginCapabilities> = Vec::new();
        
        for entry in glob(&glob_search_path).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let caps = self.load_plugin_caps(String::from(path.to_str().unwrap()))?;
                    result_vec.push(caps);
                },
                Err(e) => println!("{:?}", e),
            }
        }

        if result_vec.len()==0 {
            return Err(CuckooMinerError::NoPluginsFoundError(
                format!("No plugins found in plugin directory {}", lib_full_path.clone())
                ));
        }

        Ok(result_vec)
    }
}