use std::{iter::Cycle, vec::IntoIter};

use amethyst::{
    assets::{AssetStorage, Loader},
    audio::{output::Output, OggFormat, Source, SourceHandle, AudioSink},
    ecs::{World, WorldExt},
};

const MUSIC_TRACKS: &[&str] = &[
    "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
    "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
];

pub struct Sounds {
    pub score_sfx: SourceHandle,
    pub bounce_sfx: SourceHandle,
}

fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

pub fn initialize_audio(world: &mut World) {
    let (sound_effects, music) = {
        let loader = world.read_resource::<Loader>();

        let mut sink = world.write_resource::<AudioSink>();
        sink.set_volume(0.25);

        let music = MUSIC_TRACKS
            .iter()
            .map(|file| load_audio_track(&loader, &world, file))
            .collect::<Vec<_>>()
            .into_iter()
            .cycle();

        (
            Sounds {
                bounce_sfx: load_audio_track(&loader, &world, "audio/bounce.ogg"),
                score_sfx: load_audio_track(&loader, &world, "audio/score.ogg"),
            },
            Music { music },
        )
    };

    world.insert(sound_effects);
    world.insert(music);
}

impl Sounds {
    pub fn play_bounce(&self, storage: &AssetStorage<Source>, output: Option<&Output>) {
        if let Some(output) = output {
            if let Some(sound) = storage.get(&self.bounce_sfx) {
                output.play_once(sound, 1.0);
            }
        }
    }

    pub fn play_score(&self, storage: &AssetStorage<Source>, output: Option<&Output>) {
        if let Some(output) = output {
            if let Some(sound) = storage.get(&self.score_sfx) {
                output.play_once(sound, 1.0);
            }
        }
    }
}

pub struct Music {
    pub music: Cycle<IntoIter<SourceHandle>>,
}
