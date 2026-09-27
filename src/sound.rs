use rodio::{
    Player,
    source::SquareWave,
    stream::{DeviceSinkBuilder, MixerDeviceSink},
};

pub struct Speaker {
    _stream: Option<MixerDeviceSink>,
    player: Option<Player>,
}

impl Speaker {
    pub fn new() -> Self {
        if let Ok(mut stream) = DeviceSinkBuilder::open_default_sink() {
            stream.log_on_drop(false);
            let player = Player::connect_new(stream.mixer());
            player.set_volume(0.2); // Comfortable volume level
            player.append(SquareWave::new(440.0));
            player.pause();

            Speaker {
                _stream: Some(stream),
                player: Some(player),
            }
        } else {
            Speaker {
                _stream: None,
                player: None,
            }
        }
    }

    pub fn play(&self) {
        if let Some(player) = &self.player {
            player.play();
        }
    }

    pub fn pause(&self) {
        if let Some(player) = &self.player {
            player.pause();
        }
    }
}

impl Default for Speaker {
    fn default() -> Self {
        Self::new()
    }
}
