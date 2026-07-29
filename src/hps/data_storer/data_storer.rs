use crate::hps::custom_values::hpspuzzledata::HPSPuzzleData;
use crate::{
    hps::{
        builtins::{circleguy_builtins, circleguy_hps_builtins, loading_builtins},
        custom_values::hpspuzzle::HPSPuzzle,
        data_storer::{
            def_entry::DefEntry, io::*, keybind_data::KeybindData, puzzle_io::PuzzleIOData,
        },
    },
    puzzle::puzzle::Puzzle,
};
use hyperpuzzlescript::{
    BUILTIN_SPAN, CustomValue, EvalCtx, FnValue, FullDiagnostic, List, Map, Runtime, Scope, Spanned,
};
use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
pub type PuzzlesMap = Arc<Mutex<DefEntry>>;

#[derive(Debug)]
///stores the data for loading puzzles (definitions and basic info for preview)
pub struct DataStorer {
    pub puzzles: PuzzlesMap,
    pub rt: Runtime,
    pub keybinds: KeybindData,
}

#[derive(Debug, Clone)]
pub struct PuzzleLoadingData {
    pub name: String,
    pub path: PathBuf,
    pub authors: Vec<String>,
    pub scramble: usize,
    pub constructor: Spanned<Arc<FnValue>>,
}

#[derive(Debug, Clone)]
pub struct PuzzleData {
    pub data: HPSPuzzleData,
    pub keybinds: HashMap<egui::Key, (String, isize)>,
    pub path: PathBuf,
}

impl PuzzleLoadingData {
    pub fn load(
        &self,
        rt: &mut Runtime,
        keybinds: HashMap<egui::Key, (String, isize)>,
    ) -> Result<PuzzleData, FullDiagnostic> {
        let mut scope = Scope::default();
        scope.special.puz = HPSPuzzle::new().clone().at(BUILTIN_SPAN);
        let arc_scope = Arc::new(scope);
        self.constructor.0.call(
            self.constructor.1,
            &mut EvalCtx {
                scope: &arc_scope,
                runtime: rt,
                caller_span: BUILTIN_SPAN,
                exports: &mut None,
                stack_depth: 0,
            },
            List::new(),
            Map::new(),
        )?;
        let mut puz = arc_scope
            .special
            .puz
            .as_ref::<HPSPuzzle>()?
            .0
            .lock()
            .unwrap();
        puz.authors = self.authors.clone();
        puz.name = self.name.clone();
        puz.scramble = self.scramble;
        Ok(PuzzleData {
            data: puz.clone(),
            keybinds,
            path: self.path.clone(),
        })
    }
}

impl DataStorer {
    pub fn new(exp: bool) -> Result<Self, FullDiagnostic> {
        let mut rt = Runtime::new();
        rt.with_builtins(circleguy_hps_builtins)?;
        rt.with_builtins(circleguy_builtins)?;
        let puzzles = DefEntry::Folder((OsString::from("Definitions"), HashMap::new()));
        let puzzles_arc = Arc::new(Mutex::new(puzzles));
        let mut ds = Self {
            puzzles: puzzles_arc.clone(),
            rt,
            keybinds: KeybindData::new(),
        };
        loading_builtins(&mut ds.rt, puzzles_arc.clone(), exp).unwrap();
        Ok(ds)
    }
    pub fn reset(&mut self, exp: bool) -> Result<(), FullDiagnostic> {
        *self = Self::new(exp)?;
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_puzzles(&mut self, def_path: &str) -> Result<(), ()> {
        self.rt.modules.add_from_directory(Path::new(def_path));
        self.rt.exec_all_files();
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_keybinds(&mut self, kb_path: &str) -> Result<(), ()> {
        let data = read_file_to_string(kb_path).ok().ok_or(())?;
        self.keybinds = KeybindData::load_from_string(data).ok_or(())?;
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_save(&mut self, path: &str) -> Option<Puzzle> {
        Puzzle::from_io_data(
            PuzzleIOData::from_string(
                read_file_to_string(&format!("Puzzles/Logs/{}.kdl", path)).ok()?,
            )?,
            self,
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self, path: &str, puzzle: &Puzzle) -> Result<(), String> {
        write_string_to_file(
            &PathBuf::from(&format!("Puzzles/Logs/{}.kdl", path)),
            &puzzle.to_io_data().to_string(),
        )
        .ok()
        .ok_or("Error saving file!".to_string())
    }
    #[cfg(target_arch = "wasm32")]
    pub fn load_puzzles(&mut self, def_path: &str) -> Result<(), ()> {
        self.add_from_dir(&crate::PUZZLE_DEFINITIONS)?;
        self.rt.exec_all_files();
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    fn add_from_dir(&mut self, dir: &include_dir::Dir) -> Result<(), ()> {
        for entry in dir.entries() {
            if let Some(f) = entry.as_file()
                && let Some(ext) = f.path().extension()
                && ext == "hps"
                && let Some(cont) = f.contents_utf8()
            {
                self.rt.modules.add_file(f.path(), cont);
            } else if let Some(d) = entry.as_dir() {
                self.add_from_dir(d)?;
            }
        }
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    pub fn load_keybinds(&mut self, _kb_path: &str) -> Result<(), ()> {
        let data = crate::KEYBINDS
            .entries()
            .get(0)
            .ok_or(())?
            .as_file()
            .ok_or(())?
            .contents_utf8()
            .ok_or(())?;
        self.keybinds = KeybindData::load_from_string(data.to_string()).ok_or(())?;
        Ok(())
    }
}
