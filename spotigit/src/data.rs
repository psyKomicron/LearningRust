use std::{io::BufWriter, fs::{File, self}};

use regex::Regex;
use rspotify::model::{FullPlaylist, PlaylistItem};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct PlaylistFile
{
    pub playlist_name: String,
    pub items: Vec<PlaylistItem>,
    pub playlist_id: String
}


pub fn cache_playlist(playlist: &FullPlaylist) -> Result<(), ()>
{
    let file = File::create(format!("./{}.json", playlist.name)).map_err(|err|
    {
        eprintln!("Failed to create playlist cache file: {err}");
    })?;
    
    let playlist_file = PlaylistFile
    { 
        playlist_name: playlist.name.to_owned(), 
        items: playlist.tracks.items.to_owned(),
        playlist_id: playlist.id.to_string()
    };
    serde_json::to_writer(BufWriter::new(file), &playlist_file).map_err(|err| 
    {
        eprintln!("Failed to write playlist cache file: {err}");
    })?;

    println!("Cloned playlist.");
    Ok(())
}

pub fn open_playlist(path: &str) -> Result<PlaylistFile, std::io::Error>
{
    let mut actual_path = String::from(path);
    let file = File::open(path).or_else(|err|
    {
        eprintln!("Failed to open {path}: {err}");
        println!("Trying to find a file matching '{path}'");

        let re = Regex::new(path).expect("Provided regex isn't valid.");
        for res in fs::read_dir("./")? 
        {
            match res
            {
                Ok(entry) =>
                {
                    if re.is_match(entry.file_name().to_str().unwrap())
                    {
                        actual_path = entry.path().to_str().unwrap().to_owned();
                        return File::open(&actual_path);
                    }
                }
                Err(err) => eprintln!("Error reading './': {:#?}", err)
            }
        }

        panic!("No files match r'{path}'");
    })?;

    let playlist: PlaylistFile = serde_json::from_reader(file).unwrap_or_else(|err|
    {
        eprintln!("Failed to deserialize playlist: {err}");
        PlaylistFile 
        { 
            playlist_name: String::new(), 
            items: Vec::new(),
            playlist_id: String::new()
        }
    });

    Ok(playlist)
}