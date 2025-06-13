# qbit-renamer

Simple web-server to automatically offset numbers on torrents.

## Setup

### Docker

```yaml
services:
  qbit-renamer:
    image: secozzi/qbit-renamer:latest
    container_name: qbit-renamer
    environment:
      - QBIT_URL=https://qbittorent.example.com # (Optional) Address of qBittorrent webui, defaults to http://localhost:8080
      - PORT=3000 # (Optional) Port to run server on, defaults to 3000
      - QBIT_USERNAME=admin # (Optional)
      - QBIT_PASSWORD=password # (Optional)
```

If `QBIT_USERNAME` isn't set, qbit-renamer will connect to qBittorrent without authenticating.

### qBittorrent

In qBittorrent's settings, go to downloads and enter in `curl -X POST --data-urlencode "hash=%I" --data-urlencode "tag=%G" <url>/rename` under "Run on torrent finished", where `<url>` is the location to the server running qbit-renamer, for example `http://localhost:3000`.

## Usage

To offset the numbering for torrents, the torrent must have a tag of a specific form: `<regex>@<offset>` where `<regex>` is the regex pattern that will match the filename and `<offset>` is the offset set. `<regex>` must contain one group which matches the number in the filename that the offset will be applied to.

### Example

Let's say the filename looks like `[SubsPlease] Kusuriya no Hitorigoto - 45 (1080p) [71693351].mkv`. To subtract the episode number by 24, add the following tag: `- (\d+) \(1080@-24`. Upon completion, the file will be renamed to `[SubsPlease] Kusuriya no Hitorigoto - 21 (1080p) [71693351].mkv`.

## Known limitations

qbit-renamer has only been designed and tested for torrent that contains a single file. It will also currently not work for torrents with multiple tags.