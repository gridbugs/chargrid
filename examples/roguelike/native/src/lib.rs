use prototty_file_storage::IfDirectoryMissing;
pub use prototty_file_storage::{FileStorage, Storage};
use prototty_native_audio::{Error as NativeAudioError, NativeAudioPlayer};
use rip_prototty::{Controls, GameConfig, Omniscient, RngSeed};
pub use simon;
use simon::*;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

const DEFAULT_SAVE_FILE: &str = "save";
const DEFAULT_NEXT_TO_EXE_SAVE_DIR: &str = "save";
const DEFAULT_NEXT_TO_EXE_CONTROLS_FILE: &str = "controls.json";

pub struct NativeCommon {
    pub rng_seed: RngSeed,
    pub save_file: String,
    pub file_storage: FileStorage,
    pub controls: Controls,
    pub audio_player: Option<NativeAudioPlayer>,
    pub game_config: GameConfig,
}

fn read_controls_file(path: &PathBuf) -> Option<Controls> {
    let mut buf = Vec::new();
    let mut f = File::open(path).ok()?;
    f.read_to_end(&mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}

impl NativeCommon {
    pub fn arg() -> impl Arg<Item = Self> {
        args_map! {
            let {
                rng_seed = opt::<u64>("r", "rng-seed", "rng seed to use for first new game", "INT")
                    .option_map(|seed| RngSeed::U64(seed))
                    .with_default(RngSeed::Random);
                save_file = opt("s", "save-file", "save file", "PATH")
                    .with_default(DEFAULT_SAVE_FILE.to_string());
                save_dir = opt("d", "save-dir", "save dir", "PATH")
                    .with_default(DEFAULT_NEXT_TO_EXE_SAVE_DIR.to_string());
                controls_file = opt::<String>("c", "controls-file", "controls file", "PATH");
                delete_save = flag("", "delete-save", "delete save game file");
                omniscient = flag("", "omniscient", "enable omniscience").some_if(Omniscient);
            } in {{
                let controls_file = if let Some(controls_file) = controls_file {
                    controls_file.into()
                } else {
                    env::current_exe().unwrap().parent().unwrap().join(DEFAULT_NEXT_TO_EXE_CONTROLS_FILE)
                        .to_path_buf()
                };
                let controls = read_controls_file(&controls_file).unwrap_or_else(Controls::default);
                let mut file_storage = FileStorage::next_to_exe(
                    &save_dir,
                    IfDirectoryMissing::Create,
                ).expect("failed to open directory");
                if delete_save {
                    let result = file_storage.remove(&save_file);
                    if result.is_err() {
                        log::warn!("couldn't find save file to delete");
                    }
                }
                let audio_player = match NativeAudioPlayer::try_new_default_device() {
                    Ok(audio_player) => Some(audio_player),
                    Err(NativeAudioError::NoOutputDevice) => {
                        log::warn!("no output audio device - continuing without audio");
                        None
                    }
                };
                let game_config = GameConfig { omniscient };
                Self {
                    rng_seed,
                    save_file,
                    file_storage,
                    controls,
                    audio_player,
                    game_config,
                }
            }}
        }
    }
}
