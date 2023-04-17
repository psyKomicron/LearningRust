pub mod helpers;
pub mod data;

use std::env;
use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::result::Result;

use chrono::{self, Utc, DateTime};

use console::style;
use dialoguer::{Confirm, Input};
use tokio;
use rspotify::{prelude::*, ClientCredsSpotify, Credentials};
use rspotify::model::{PlayableItem, SimplifiedPlaylist, SearchResult, SearchType, AlbumId, PlaylistItem, PlaylistId};

use helpers::*;
use data::*;


async fn albums(spotify: &ClientCredsSpotify)
{
    // Running the requests
    let birdy_uri = AlbumId::from_uri("spotify:album:0sNOF9WDwhWunNAHPD3Baj").unwrap();
    let albums = spotify.album(birdy_uri).await.unwrap();

    println!("tracks:");
    for track in albums.tracks.items
    {
        println!("\ttrack: {} / {} / http://open.spotify.com/track/{}", 
            track.name, 
            track.preview_url.unwrap_or(track.href.unwrap_or("".to_string())),
            track.id.unwrap().id()
        );
    }
}

async fn search_playlist(query: &str, offset: u32, spotify: &ClientCredsSpotify) -> Result<Vec<SimplifiedPlaylist>, ()>
{
    let res = spotify.search(
            &format!("{}", query), 
            SearchType::Playlist, 
            None, 
            None, 
            Some(10), 
            Some(offset * 10))
        .await;

    if let Ok(search_result) = res
    {
        match search_result
        {
            SearchResult::Playlists(playlist_page) => 
            {
                let vec = playlist_page.items.to_vec();
                return Ok(vec);
            },
            _ =>
            {
                todo!("Better error implementation.");
            }
        }
    }   
    todo!("Better error implementation.");
}

async fn clone_playlist(playlist_id: &PlaylistId<'_>, spotify: &ClientCredsSpotify)
{
    let fullplaylist = spotify.playlist(playlist_id.clone_static(), None, None).await.unwrap();

    println!("Cloning {} by {}", fullplaylist.name, fullplaylist.owner.display_name.clone().unwrap_or(format!("")));
    let mut index = 1;
    for item in &fullplaylist.tracks.items
    {
        // if let PlayableItem::Track(track) = item.track.as_ref().unwrap()
        // {
        //     let names = track.artists.iter().map(|artist| 
        //     {
        //         artist.name.clone()
        //     }).collect::<Vec<String>>();
        //     //println!("  [{index}] '{}' by {}", track.name, names.join(", "));
        //     print_playable_item(item, &format!("  []{index}]"));
        //     index += 1;
        // }
        print_playable_item(&item.track, &format!("  {index}."));
        index += 1;
    }

    if Confirm::new().with_prompt("Do you want to clone this playlist ?").interact().unwrap()
    {
        _ = cache_playlist(&fullplaylist).map_err(|err|
            {
                eprintln!("Failed to cache playlist: {err:#?}");
            });
    }
    else 
    {
        println!("Not cloning {}", fullplaylist.name);
    }
}

fn lazy_eval(left: &PlaylistItem, right: &PlaylistItem) -> bool
{
    if let Some(left_item) = &left.track
    {
        if let Some(right_item) = &right.track
        {
            match left_item
            {
                PlayableItem::Track(left_track) => 
                {
                    if let PlayableItem::Track(right_track) = right_item
                    {
                        let left_song_name = left_track.name.clone();
                        let left_artists_name = left_track.artists.iter().map(|artist| 
                            {
                                artist.name.clone()
                            })
                            .collect::<Vec<String>>();

                        let right_song_name = right_track.name.clone();
                        let right_artists_name = right_track.artists.iter().map(|artist| 
                            {
                                artist.name.clone()
                            })
                            .collect::<Vec<String>>();

                        if left_song_name != right_song_name
                        {
                            //println!("Difference: songs do not have the same name.");
                            return false;
                        }

                        if left_track.duration != right_track.duration
                        {
                            println!("Difference: songs aren't same length ({} & {}).", left_song_name, right_song_name);
                            return false;
                        }
                        
                        if left_track.album.name != right_track.album.name
                        {
                            println!("Difference: songs don't belong to the same album.");
                            return false;
                        }

                        if left_artists_name.len() != right_artists_name.len()
                        {
                            println!("Difference: artists names are not the same length.");
                            return false;
                        }
                        
                        for i in 0..left_artists_name.len()
                        {
                            if left_artists_name[i] != right_artists_name[i]
                            {
                                println!("Difference: artist at {i} {} != {}", left_artists_name[i], right_artists_name[i]);
                                return false;
                            }
                        }

                        return true;
                    }
                    else 
                    {
                        return false;
                    }
                },
                PlayableItem::Episode(_) =>
                {
                    unimplemented!("Not yet done for episodes.");
                },
            }
        }
    }

    return false;   
}

async fn clone(query: &str, spotify: &ClientCredsSpotify)
{
    let local_query = String::from(query);
    let mut offset: u32 = 0;
    let re = regex::Regex::new("[0-9A-z]{22}").unwrap();
    if re.is_match(&query)
    {
        let s = format!("spotify:playlist:{query}");
        let id = PlaylistId::from_uri(&s).unwrap();
        clone_playlist(&id, spotify).await;
    }
    else 
    {
        loop 
        {
            match search_playlist(&local_query, offset, &spotify).await
            {
                Ok(res) => 
                {
                    let mut iter = 0;
                    let mut names: Vec<String> = res.iter().map(|item|
                        {
                            iter += 1;
                            format!("{}. '{}' by {}", iter, item.name, item.owner.display_name.as_ref().unwrap_or(&format!("")))
                        }).collect();
                    names.insert(0, format!("Continue"));

                    match dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .items(&names)
                        .with_prompt("Select a playlist")
                        .default(0)
                        .interact()
                    {
                        Ok(mut index) =>
                        {
                            if index == 0
                            {
                                offset += 1;
                                continue;
                            }
                            else 
                            {
                                index -= 1;
                                let i: usize = index.try_into().unwrap();
                                if i >= names.len()
                                {
                                    eprintln!("Index not in range.");
                                }
                                else 
                                {
                                    clone_playlist(&res[i].id, spotify).await;
                                }   

                                break;
                            }
                        },
                        Err(err) => eprintln!("{:#}", err),
                    }
                },
                Err(err) => eprintln!("{err:#?}")
            }    
        }
    }
}

async fn diff(playlist_file: &PlaylistFile, spotify: &ClientCredsSpotify)
{
    let id = PlaylistId::from_uri(&playlist_file.playlist_id).unwrap();
    match spotify.playlist(id, None, None).await
    {
        Ok(full_playlist) =>
        {
            let mut removed = 0;
            //let mut added = 0;
            
            for i in 0..playlist_file.items.len()
            {
                let mut contains = false;
                let mut same_index = false;
                let mut index: usize = playlist_file.items.len();
                for j in 0..full_playlist.tracks.items.len()
                {
                    if lazy_eval(&playlist_file.items[i], &full_playlist.tracks.items[j])
                    {
                        contains = true;
                        same_index = i == j;
                        index = j;
                        break;
                    }
                }

                if contains
                {
                    if removed > 0
                    {
                        let n: i32 = (i as i32) - removed;
                        if n > 0 && (n as usize) < full_playlist.tracks.items.len()
                        {       
                            if lazy_eval(&playlist_file.items[i], &full_playlist.tracks.items[n as usize])
                            {
                                // The tracks are the same, just moved by another track being added or removed.
                                continue;
                            }
                        }
                    }

                    if !same_index
                    {
                        print_playable_item(&playlist_file.items[i].track, &format!(" {} {}", 
                            style("*").yellow().bold(),
                            style(format!("Track {} moved to {}:", i + 1, index + 1)).bold(),
                        ));
                    }
                    // else unchanged.
                }
                else
                {
                    if index == playlist_file.items.len()
                    {
                        print_playable_item(
                            &playlist_file.items[i].track, &format!(" {} {}",
                                style(">").cyan().bold(),
                                style(format!("Track {} added:", i + 1)).bold()
                            )
                        );
                    }
                    else 
                    {
                        print_playable_item(&full_playlist.tracks.items[index].track, &format!(" {} {}", 
                            style("-").red().bold(),
                            style(format!("Track {} removed:", i + 1)).bold(),
                        ));
                        removed += 1;
                    }
                }
            }

            for i in 0..full_playlist.tracks.items.len()
            {
                let mut contains = false;
                for j in 0..playlist_file.items.len()
                {
                    if lazy_eval(&playlist_file.items[j], &full_playlist.tracks.items[i])
                    {
                        contains = true;
                        break;
                    }
                }

                if !contains
                {
                    // Track is new.
                    print_playable_item(
                        &full_playlist.tracks.items[i].track, &format!(" {} {}",
                            style("+").green().bold(),
                            style(format!("Track {} new:", i + 1)).bold()
                        )
                    );
                    // added += 1;
                }
            }
        },
        Err(err) => eprintln!("{:#?}", err)
    }
}

async fn add(query: &str, spotify: &ClientCredsSpotify)
{
    // Search for the song.
    match spotify.search(query, SearchType::Track, None, None, Some(10), Some(0)).await
    {
        Ok(search_result) =>
        {
            match search_result
            {
                SearchResult::Tracks(track_page) =>
                {
                    let items = track_page.items
                        .iter()
                        .map(|item|
                            {
                                format!("{} - {} | {}", &item.name, &item.album.name, &item.artists.iter().map(|artist| artist.name.clone()).collect::<Vec<String>>().join(", "))
                            }).collect::<Vec<String>>();

                    match dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("Select a song")
                        .items(&items)
                        .interact()
                    {
                        Ok(index) =>
                        {
                            println!("Selected track: {}", format_track(&track_page.items[index]));

                            if let Ok(input) = Input::<String>::new()
                                    .with_prompt("Playlist name ?")
                                    .interact_text()
                            {
                                match open_playlist(&input)
                                {
                                    Ok(mut playlist_file) =>
                                    {
                                        println!("Selected playlist: {}", &playlist_file.playlist_name);
                                        let playlist_item = PlaylistItem
                                        {
                                            added_at: Some(chrono::offset::Utc::now()),
                                            is_local: false,
                                            added_by: None,
                                            track: Some(PlayableItem::Track(track_page.items[index].clone()))
                                        };
                                        playlist_file.items.push(playlist_item);

                                        println!("Added {} to {}", format_track(&track_page.items[index]), &playlist_file.playlist_name);
                                        display_playlist(&playlist_file.items, spotify).await;
                                        
                                        let file = OpenOptions::new()
                                            .append(false)
                                            .create(false)
                                            .write(true)
                                            .open(format!("./{}.json", playlist_file.playlist_name))
                                            .expect("Couldn't open playlist file.");

                                        _ = serde_json::to_writer(BufWriter::new(file), &playlist_file).map_err(|err| 
                                        {
                                            eprintln!("Failed to write playlist cache file: {err}");
                                        });
                                    },
                                    Err(err) => eprintln!("Could not open playlist: {:#}", err)
                                }
                            }
                        },
                        Err(err) => eprintln!("{:#}", err),
                    }
                    
                },
                _ => todo!("That is not a track.")
            }
        },
        Err(err) =>
        {
            println!("Failed to search track ('{query}'): {:#}", err);
        }
    }
}


#[tokio::main]
async fn main() 
{
    println!("{}{} by {}{}{}",
        style("spoti").green().bold(),
        style("git").red().bold(),
        style("psy").blue().bold(),
        style("K").bright().bold(),
        style("omicron").red().bold()
    );
    
    let mut args = env::args();
    _ = args.next();
    let command = args.next().unwrap_or_default();
    let collection: Vec<String> = env::args().collect();
    if collection.len() < 3
    {
        eprintln!("Not enough args");
        return;
    }
    let query = &collection[2..].join(" ");
    

    let id = env::var("SPOTIFY_ID").expect("Spotify application Id not found.");
    let secret = env::var("SPOTIFY_TOKEN").expect("Spotify application token not found.");

    let creds = Credentials{ id, secret: Some(secret) };
    let spotify = ClientCredsSpotify::new(creds);
    spotify.request_token().await.expect("Failed to initialize Spotify client credentials.");

    match command.as_str()
    {
        "clone" => 
        {
            let mut query = args.next().unwrap_or("".to_string());
            if query == ""
            {
                eprintln!("Query is empty.");
                if let Ok(input) = Input::<String>::new()
                        .with_prompt("Playlist name ?")
                        .interact_text()
                {
                    query = input; // ?
                }
            }

            clone(&query, &spotify).await;
        },

        "search" =>
        {
            eprintln!("Not yet implemented :)");
        }

        "diff" =>
        {
            let playlist_file = open_playlist(&query).unwrap();
            diff(&playlist_file, &spotify).await;
        }

        "show" =>
        {
            let playlist = open_playlist(&query).unwrap();
            println!("{}", &playlist.playlist_name);
            display_playlist(&playlist.items, &spotify).await;
        }

        "add" =>
        {
            println!("{}", style("Add is unstable.").yellow());
            add(query, &spotify).await;
        },

        "album" => albums(&spotify).await,

        "" =>
        {
            println!("{}", style("Command required: use clone, show, diff, search.").yellow().bold());
        },

        _ => println!("Option not recognized.")
    }
}
