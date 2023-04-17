use console::style;
use rspotify::{model::{PlaylistItem, PlayableItem, FullTrack}, ClientCredsSpotify, prelude::BaseClient};

pub struct ItemName
{
    song_name: String,
    artist_name: Vec<String>,
    album_name: String
}

pub async fn display_playlist(items: &Vec<PlaylistItem>, spotify: &ClientCredsSpotify)
{
    let mut max = [0, 0, 0];
    for item in items
    {
        let item_name = get_playable_item_name(&item.track);

        if usize::max(item_name.song_name.chars().count(), item_name.album_name.chars().count()) > max[0]
        {
            max[0] = usize::max(item_name.song_name.chars().count(), item_name.album_name.chars().count());
        }

        if item_name.artist_name.join(", ").chars().count() > max[1]
        {
            max[1] = item_name.artist_name.join(", ").chars().count();
        }
    }

    let mut index = 1;
    for item in items
    {
        let item_name = get_playable_item_name(&item.track);
        let mut song_name = item_name.song_name.clone();
        let mut artist_name = item_name.artist_name.join(", ");

        if song_name.chars().count() < max[0]
        {
            song_name = format!("{song_name}{}", " ".repeat(max[0] - song_name.chars().count()));
        }
        if artist_name.chars().count() < max[1]
        {
            artist_name = format!("{artist_name}{}", " ".repeat(max[1] - artist_name.chars().count()));
        }

        let mut padding_index = format!("{index}");
        let padding = f32::log10(items.len() as f32);
        let n = padding_index.len() as f32;
        if n < padding 
        {
            padding_index = format!("{}{index}", " ".repeat((padding - n).ceil() as usize));
        }

        let mut display_name: Option<String> = None;
        if let Some(user) = &item.added_by
        {
            match spotify.user(user.id.clone()).await
            {
                Ok(public_user) => display_name = public_user.display_name,
                Err(_) => {}//eprintln!("Failed to fetch user: {:#}", err)
            }
        }
        
        println!(
            " {}. {}  {}  {}\n{}\n",
            padding_index,
            style(song_name).bold(), 
            artist_name, 
            format!(
                "{} by {}",
                item.added_at.map_or_else(|| format!("") ,|date| format!("added {}", date.format("%d/%m/%Y"))),
                display_name.unwrap_or(String::from("Anonymous"))
            ),
            style(
                format!("{}{}", " ".repeat(padding_index.len() + 3), &item_name.album_name)
            ).dim().italic()
        );
        index += 1;
    }
}

pub fn format_track(track: &FullTrack) -> String
{
    let song_name = track.name.clone();
    let artists_name = track.artists.iter().map(|artist| 
        {
            artist.name.clone()
        })
        .collect::<Vec<String>>()
        .join(", ");

    format!("{} - {}", 
        style(song_name).bright(),
        style(artists_name).dim().italic()
    )
}

pub fn get_playable_item_name(option: &Option<PlayableItem>) -> ItemName
{
    if let Some(playable_item) = option
    {
        if let PlayableItem::Track(track) = playable_item
        {
            let song_name = track.name.clone();
            let artist_name = track.artists.iter().map(|artist| 
                {
                    artist.name.clone()
                })
                .collect::<Vec<String>>();
            let album_name = track.album.name.to_owned();
            return ItemName{ song_name, artist_name, album_name };
        }
        else if let PlayableItem::Episode(episode) = playable_item
        {
            return ItemName{ song_name: episode.name.clone(), artist_name: Vec::new(), album_name: String::new() };
        }
    }
    return ItemName{ song_name: format!(""), artist_name: Vec::new(), album_name: String::new() };
}

pub fn print_playable_items(v1: &Option<PlayableItem>, v2: &Option<PlayableItem>, message: &str)
{
    let item_name_left = get_playable_item_name(v1);
    let item_name_right = get_playable_item_name(v2);
    let names_left = item_name_left.artist_name
        .iter().map(|name|
        {
            style(name).italic().to_string()
        })
        .collect::<Vec<String>>()
        .join(",");
    let names_right = item_name_right.artist_name.join(",");

    println!("{} {} - {} -> {} - {}", 
        message,
        style(item_name_left.song_name).bright(),
        style(names_left).dim().italic(),
        style(item_name_right.song_name).bright(),
        style(names_right).dim().italic()
    );
}

pub fn print_playable_item(v1: &Option<PlayableItem>, message: &str)
{
    let item_name_left = get_playable_item_name(v1);
    let names_left = item_name_left.artist_name.join(",");

    println!("{} {} - {}", 
        message,
        style(item_name_left.song_name).bright(),
        style(names_left).dim().italic(),
    );
}