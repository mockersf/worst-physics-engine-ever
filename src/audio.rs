use bevy::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_audio)
            .add_systems(Update, play_audio)
            .add_event::<AudioEvent>();
    }
}

#[derive(Event)]
pub enum AudioEvent {
    Click,
    Jump,
    Fall,
    Win,
    Crash,
    AddCollider,
    RemoveCollider,
    FailedCollider,
    Eagle,
}

#[derive(Resource)]
struct AudioHandles {
    click: Handle<AudioSource>,
    jump: Handle<AudioSource>,
    fall: Handle<AudioSource>,
    win: Handle<AudioSource>,
    crash: Handle<AudioSource>,
    add_collider: Handle<AudioSource>,
    remove_collider: Handle<AudioSource>,
    failed_collider: Handle<AudioSource>,
    eagle: Handle<AudioSource>,
}

fn load_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AudioHandles {
        click: asset_server.load("click.ogg"),
        jump: asset_server.load("jump.ogg"),
        fall: asset_server.load("fall.ogg"),
        win: asset_server.load("win.ogg"),
        crash: asset_server.load("crash.ogg"),
        add_collider: asset_server.load("add.ogg"),
        remove_collider: asset_server.load("remove.ogg"),
        failed_collider: asset_server.load("failed.ogg"),
        eagle: asset_server.load("eagle.ogg"),
    });
}

fn play_audio(
    mut events: EventReader<AudioEvent>,
    mut commands: Commands,
    handles: Res<AudioHandles>,
) {
    for event in events.read() {
        match event {
            AudioEvent::Click => {
                commands.spawn(AudioBundle {
                    source: handles.click.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            AudioEvent::Jump => {
                commands.spawn(AudioBundle {
                    source: handles.jump.clone(),
                    settings: PlaybackSettings::DESPAWN
                        .with_volume(bevy::audio::Volume::new_relative(0.5)),
                });
            }
            AudioEvent::Fall => {
                commands.spawn(AudioBundle {
                    source: handles.fall.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            AudioEvent::Win => {
                commands.spawn(AudioBundle {
                    source: handles.win.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            AudioEvent::Crash => {
                commands.spawn(AudioBundle {
                    source: handles.crash.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            AudioEvent::AddCollider => {
                commands.spawn(AudioBundle {
                    source: handles.add_collider.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            AudioEvent::RemoveCollider => {
                commands.spawn(AudioBundle {
                    source: handles.remove_collider.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            AudioEvent::FailedCollider => {
                commands.spawn(AudioBundle {
                    source: handles.failed_collider.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            AudioEvent::Eagle => {
                commands.spawn(AudioBundle {
                    source: handles.eagle.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
        }
    }
}
