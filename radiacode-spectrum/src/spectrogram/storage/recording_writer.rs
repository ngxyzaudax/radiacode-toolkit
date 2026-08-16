use std::fs::{self, File};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::spectrogram::model::{SpectrogramHeader, SpectrogramRow, SpectrogramSeries};

use super::recording_load::load_recording;
use super::storage_format::{MAGIC, VERSION_CURRENT, VERSION_V2, read_u32, write_f64, write_u32};

const ROW_COUNT_FLUSH_INTERVAL: u32 = 10;

pub struct RecordingWriter {
    file: BufWriter<File>,
    row_count_offset: u64,
    pub path: PathBuf,
    pub row_count: u32,
    version: u32,
}

impl RecordingWriter {
    pub fn create(path: PathBuf, header: &SpectrogramHeader) -> std::io::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);
        let header_json = serde_json::to_vec(header)?;
        writer.write_all(MAGIC)?;
        write_u32(&mut writer, VERSION_CURRENT)?;
        write_u32(&mut writer, header_json.len() as u32)?;
        writer.write_all(&header_json)?;
        write_u32(&mut writer, header.channel_count)?;
        let row_count_offset = writer.stream_position()?;
        write_u32(&mut writer, 0)?;
        writer.flush()?;
        Ok(Self {
            file: writer,
            row_count_offset,
            path,
            row_count: 0,
            version: VERSION_CURRENT,
        })
    }

    pub fn append_row(&mut self, row: &SpectrogramRow) -> std::io::Result<()> {
        for value in &row.counts {
            write_u32(&mut self.file, *value)?;
        }
        if self.version >= VERSION_V2 {
            write_f64(&mut self.file, row.interval_secs)?;
            self.file.write_all(&[row.kind.storage_tag()])?;
            write_f64(&mut self.file, row.kind.storage_extra())?;
        }
        self.row_count += 1;
        self.file.flush()?;
        if self.row_count.is_multiple_of(ROW_COUNT_FLUSH_INTERVAL) {
            self.write_row_count_header()?;
        }
        Ok(())
    }

    pub fn finalize(mut self) -> std::io::Result<PathBuf> {
        self.write_row_count_header()?;
        self.file.get_mut().sync_all()?;
        Ok(self.path)
    }

    fn write_row_count_header(&mut self) -> std::io::Result<()> {
        self.file.flush()?;
        self.file
            .get_mut()
            .seek(SeekFrom::Start(self.row_count_offset))?;
        write_u32(&mut self.file, self.row_count)?;
        self.file.flush()?;
        self.file.get_mut().seek(SeekFrom::End(0))?;
        Ok(())
    }
}

pub fn open_recording_append(path: PathBuf) -> std::io::Result<RecordingWriter> {
    let series = load_recording(&path)?;
    let row_count = series.rows.len() as u32;
    let mut inner = File::open(&path)?;
    let mut magic = [0_u8; 4];
    inner.read_exact(&mut magic)?;
    let version = read_u32(&mut inner)?;
    let header_len = read_u32(&mut inner)? as usize;
    inner.seek(SeekFrom::Current(header_len as i64))?;
    let _channel_count = read_u32(&mut inner)?;
    let row_count_offset = inner.stream_position()?;
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)?;
    file.seek(SeekFrom::End(0))?;
    Ok(RecordingWriter {
        file: BufWriter::new(file),
        row_count_offset,
        path,
        row_count,
        version,
    })
}

pub fn write_recording(path: &std::path::Path, series: &SpectrogramSeries) -> std::io::Result<()> {
    let mut writer = RecordingWriter::create(path.to_path_buf(), &series.header)?;
    for row in &series.rows {
        writer.append_row(row)?;
    }
    writer.finalize()?;
    Ok(())
}
