# spotifm (1.1.0-beta)

spotifm streams your spotify music over the internet using icecast2 and spawns a rest api

> note: spotifm only works with spotify premium accounts

there is an included irc bot which can be used to control the radio

## quick start
### 1) configuration
edit `config.json.example` and rename it to `config.json` 

```
    "user": "your@email.com",
    "pass": "yourpass1",
    "uris": [
        "spotify:playlist:2WvtFSAkmcABdm3iAvYwXk"
    ],
```

`user` is your email

`pass` is your password

`uris` is a list of spotify URIs (track, album or playlist) to play once started (`spotify:track:<ID>` or `spotify:album:<ID>` or `spotify:playlist:<ID>`)

### 2) track announcments and bumpers

spotifm can announce the name of the song before it plays, as well as periodically play radio station bumpers of your choosing, configured as follows:

```
    "announce": {
        "song": {
            "enable": false,
            "fmt": "{} by {}",
            "espeak": {
                "gap": 10,
                "speed": 150,
                "pitch": 50,
                "voice": "en-us",
                "amplitude": 100
            }
        },
        "bumper": {
            "enable": false,
            "freq": 20,
            "tags": [
                "you are listening to my radio"
            ],
            "espeak": {
                "gap": 10,
                "speed": 120,
                "pitch": 50,
                "voice": "en-us",
                "amplitude": 100
            }
        }
    }
```

they are disabled by default


### 2) build spotifm

`docker compose run --rm -u builder builder`


### 3) deploy spotifm
`docker compose up -d --force-recreate streamer`

> icecast2 will become available on port `8000`

> spotifm will spawn a rest api on port `9090`

### 4) irc bot (optional)
make sure to edit `ircbot.json.example` and rename it to `ircbot.json`, then

`docker compose up -d ircbot`

## rest api endpoints
### `GET /np`
### `GET /skip`
### `GET /queue/<TRACK-ID>`
### `GET /play/<TRACK-ID>`
all return (example):
```
{
    "id": "6bu8npt0GdVeESCM7K4The",
    "rid": 1676115018281,
    "track": "Speak Up",
    "artists": [
        "Freddie Dredd"
    ]
}
``` 
or `{ "error": "<error msg>"}`
### `GET /playlist`
### `GET /shuffle`
returns (example):
```
[
    {
        "id": "6bu8npt0GdVeESCM7K4The",
        "rid": 1676118353658,
        "track": "Speak Up",
        "artists": [
            "Freddie Dredd"
        ]
    },

    ...
]
```
### `GET /search/<TRACK|ARTIST|ALBUM|PLAYLIST>/<LIMIT>?q=<QUERY>`
returns (example):
```
[
    {
        "album": { ... },
        "artists": [ ... ],
        "available_markets": [ ... ],
        "disc_number": 1,
        "duration_ms": 122331,
        "explicit": true,
        "external_ids": { ... },
        "external_urls": { ... },
        "href": "https://api.spotify.com/v1/tracks/2nzjXDv6OuRHrHKhfhfB98",
        "id": "2nzjXDv6OuRHrHKhfhfB98",
        "is_local": false,
        "name": "Low Key",
        "popularity": 62,
        "preview_url": "https://p.scdn.co/mp3-preview/c4008f1cb619949d448818891c5839404a0d53ad?cid=65b708073fc0480ea92a077233ca87bd",
        "track_number": 3
    },

    ...
]
```
or `{ "error": "<error msg>"}`
