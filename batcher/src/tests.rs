pub fn run()
{
    let urls = vec![
        "https://www.youtube.com/watch?v=ECJg6QH3Zhc".to_string(),
        "https://www.youtube.com/watch?v=OYAKjlYm_Ew".to_string(),
        "http://www.lemel.gallery/ign/Legende%20de%20carte%20IGN.pdf".to_string(),
        "https://mail.google.com/mail/u/0/#inbox/FMfcgzGsltLlNCGplGgXGHPgxrGMKrXr?projector=1&messagePartId=0.2".to_string(),
        "https://github.com/psyKomicron/Winnerino/tree/v2.1.x/Winnerino/Winnerino".to_string(),
        "https://github.com/psyKomicron/Winnerino/blob/v2.1.x/Winnerino/Winnerino/Helper.h".to_string(),
        "https://www.youtube.com/watch?v=x_9gh50CI2E".to_string(),
        "https://www.lamarinerecrute.fr/sites/default/files/orientationkit/documents/2022_MSRM_PAO_TRYPTIQUE_SPORT_C_151.pdf".to_string(),
        "https://doc.rust-lang.org/rust-by-example/mod/visibility.html".to_string(),
        "https://blog.logrocket.com/making-http-requests-rust-reqwest/".to_string()
    ];

    let _arr = &urls[0..4];
    //crate::threaded_queue::setup_queue(arr);
    for ele in urls{
        println!("{ele}");
    }
}