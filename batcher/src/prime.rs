pub fn eratosthenes(n: usize) -> Vec<i64>
{
    let mut bool_array = vec![true; n + 1];
    let max: f64 = (n as f64).sqrt();
    let mut i = 2;
    loop
    {
        if i as f64 > max
        {
            break;
        }

        if bool_array[i]
        {
            let mut j = i.pow(2);
            while j <= n
            {
                bool_array[j] = false;
                j += i;
            }
        }

        i += 1;
    }

    let mut res = Vec::<i64>::new();
    for i in 0..bool_array.len()
    {
        let b = bool_array[i];
        if b
        {
            res.push(i as i64);
        }
    }

    res
}

fn atkin_conditions(n: f64, x: f64, y: f64) -> Result<usize, ()>
{
    let m = x.powi(2).mul_add(4.0, y.powi(2));
    if m <= n && (m % 12.0 == 1.0 || m % 12.0 == 5.0)
    {
        return Ok(m as usize);
    }

    let m = x.powi(2).mul_add(3.0, y.powi(2));
    if m <= n && m % 12.0 == 7.0
    {
        return Ok(m as usize);
    }
    
    let m = x.powi(2).mul_add(4.0, -y.powi(2));
    if x > y && m <= n && m % 12.0 == 11.0
    {
        return Ok(m as usize);
    }
    
    return Err(());
}

pub fn atkin(n: usize) -> Vec<i64>
{
    let mut sieve = vec![false; n + 1];

    let max: f64 = (n as f64).sqrt();
    let mut x: f64 = 1.0;

    if n >= 2
    {
        sieve[2] = true;
    }
    if n >= 3
    {
        sieve[3] = true;
    }
    
    loop 
    {
        if x > max
        {
            break;
        }

        let mut y: f64 = 1.0;
        while y <= max
        {
            if let Ok(m) = atkin_conditions(n as f64, x, y)
            {
                sieve[m] ^= true;
            }
            y += 2.0;
        }

        x += 1.0;
    }

    for i in 5..(n + 1)
    {
        if sieve[i]
        {
            let mut j = i.pow(2);
            while j <= n
            {
                sieve[j] = false;
                j += i.pow(2);
            } 
        }
    }

    let mut res = Vec::<i64>::new();
    for i in 0..sieve.len()
    {
        let b = sieve[i];
        if b
        {
            res.push(i as i64);
        }
    }

    res
}