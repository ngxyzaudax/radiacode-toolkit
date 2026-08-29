# Frame layout

Every request and response is length-prefixed.

## Request

```text
u32 payload_len LE
u16 command LE
u8  reserved (0)
u8  sequence (0x80..0x9F, wraps mod 32)
... command args ...
```

## Response

The transport reassembles chunks until complete, then yields:

```text
i32 inner_len LE          (negative → protocol error)
u16 command LE              (echo)
u8  reserved (0)            (echo)
u8  sequence                (echo)
... body ...
```

The four echoed bytes must match the request header.

## Virtual string read body

After the echoed header:

```text
u32 retcode   (1 = OK)
u32 flen      (payload byte count)
... payload ...
```

Some firmware builds append a trailing `0x00` beyond `flen`; readers trim one NUL when `size == flen + 1`.

## Virtual string write body

```text
u32 retcode   (1 = OK)
```

## Session initialization

On connect (unless restoring a cached session):

1. `SetExchange` with payload `01 ff 12 ff`
2. `SetTime` with `[day, month, year-2000, 0, second, minute, hour, 0]`
3. Write `DeviceTime = 0`
4. Set `base_time = now + 128 s` for DataBuf wall-clock conversion
5. Read `SfrFile`, validate static catalog
6. `GetVersion`; reject firmware < 4.8 unless ignored
7. Read `Configuration`; parse `SpecFormatVersion=`

## SetTime layout

Eight bytes: day, month, year minus 2000, zero, second, minute, hour, zero.
