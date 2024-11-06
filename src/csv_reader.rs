use csv::ReaderBuilder;
use std::fs::File;
use std::error::Error;

#[derive(Debug)]
pub struct TelemetryRecord {
    pub timestamp_ns: u64,
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub vel_z: f64,
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
pub fn read_csv_and_chunk(file_path: &str, time_chunk_duration: u64) -> Result<Vec<Vec<TelemetryRecord>>, Box<dyn Error>> {
    // Open the CSV file
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut current_chunk: Vec<TelemetryRecord> = Vec::new();
    let mut previous_timestamp: Option<u64> = None;
    let mut chunks: Vec<Vec<TelemetryRecord>> = Vec::new();

    // Iterate through csv file
    for result in rdr.records() {
        let record = result?;
        // Note: failed to parse straight to the struct because of the scientific notation
        // so we parse as f64 to handle scientific notation and then cast back to u64
        let timestamp_ns: u64 = record
            .get(0)
            .unwrap()
            .parse::<f64>()?
            .round() as u64;

        // Other data columns can be handled here
        let pos_x = record.get(1).unwrap().parse::<f64>().unwrap();
        let pos_y = record.get(2).unwrap().parse::<f64>().unwrap();
        let pos_z = record.get(3).unwrap().parse::<f64>().unwrap();
        let vel_x = record.get(4).unwrap().parse::<f64>().unwrap();
        let vel_y = record.get(5).unwrap().parse::<f64>().unwrap();
        let vel_z = record.get(6).unwrap().parse::<f64>().unwrap();
        let row = TelemetryRecord { timestamp_ns, pos_x, pos_y, pos_z, vel_x, vel_y, vel_z };

        // If it's the first row, start a new chunk
        if previous_timestamp.is_none() {
            current_chunk.push(row);
            previous_timestamp = Some(timestamp_ns);
            continue;
        }

        // Check the time difference from the previous timestamp
        if let Some(prev_ts) = previous_timestamp {
            let time_diff = timestamp_ns.saturating_sub(prev_ts);

            // If time difference exceeds the chunk duration, start a new chunk
            if time_diff > time_chunk_duration {
                // push current chunk since we are done with it.
                chunks.push(current_chunk);

                // Create a new chunk to populate
                current_chunk = Vec::new();
                current_chunk.push(row);

                // Update the previous timestamp
                previous_timestamp = Some(timestamp_ns);
            } else {
                 current_chunk.push(row); // Add the row to the current chunk
            }
        }
    }

    // Add the last chunk to the vec
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_chunked_read() {
        let file_path = "data/test.csv";

        let time_chunk_duration = 1_000_000_000;
        let actual = read_csv_and_chunk(file_path, time_chunk_duration).unwrap();

        assert_eq!(3, actual.len());

        for (i, chunk) in actual.iter().enumerate() {
            assert_eq!(2, chunk.len());
        }
    }
}
