pub fn pretty_print(size: u64) -> String
{
    let exts = &["B", "KiB", "MiB", "GiB", "TiB"];
    //let mut formattedSize = size;
    let byte_size: u64 = 1024;
    let mut n = 0;
    for ext in exts
    {
        if (size / byte_size.pow(n)) < 1024
        {
            return format!("{} {}", size / byte_size.pow(n), ext);
        }
        n += 1;
    }

    return String::new();
}