use std::{collections::HashMap, time::Duration};

use crate::data_dir::DataDirError;

use super::{badge::BadgeType, ui::ScriptWidget, PocketPyScript, PocketPyScriptError};

pub struct ScriptList<T> {
	scripts: HashMap<String, ScriptEditor<T>>,
	path: std::path::PathBuf,
}

impl<T: BadgeType> ScriptList<T> {
	pub fn new() -> Result<(Self, Vec<PocketPyScriptError>), &'static DataDirError> {
		let path = T::get_path()?.to_owned();
		let mut scripts = HashMap::new();
		let mut errors = Vec::new();

		match std::fs::read_dir(&path) {
			Ok(entries) => {
				for entry in entries {
					match entry {
						Ok(entry) => match entry.metadata() {
							Ok(metadata) if metadata.is_file() => {
								match PocketPyScript::load(entry.path()) {
									Ok(script) => {
										scripts.insert(
											script.name.clone(),
											ScriptEditor {
												script: script,
												state: ScriptEditorState::DisplayMode,
												marked_for_deletion: false,
												_t: std::marker::PhantomData,
											},
										);
									}
									Err(e) => errors.push(e.into()),
								}
							}
							Ok(_) => {}
							Err(e) => errors.push(e.into()),
						},
						Err(e) => errors.push(e.into()),
					}
				}
			}
			Err(e) => errors.push(e.into()),
		}

		Ok((Self { scripts, path }, errors))
	}

	pub fn add(&mut self, script: PocketPyScript) -> Result<(), PocketPyScriptError> {
		script.save(self.path.as_path())?;
		self.scripts.insert(
			script.name.clone(),
			ScriptEditor {
				state: ScriptEditorState::EditMode(script.clone()),
				script,
				marked_for_deletion: false,
				_t: std::marker::PhantomData,
			},
		);
		Ok(())
	}

	pub fn save_all(&self) -> Vec<PocketPyScriptError> {
		let mut errors = Vec::new();

		self.scripts.values().for_each(|script| {
			if let Err(e) = script.script.save(self.path.as_path()) {
				errors.push(e);
			}
		});

		errors
	}

	pub fn cleanup(&mut self) {
		let mut name_swaps: Vec<(String, String)> = Vec::new();
		let mut deletions: Vec<String> = Vec::new();

		for (name, editor) in self.scripts.iter_mut() {
			if editor.marked_for_deletion {
				if let Err(e) = editor.script.delete(&self.path) {
					crate::toasts()
						.error(format!("Couldn't delete script: {}", e))
						.set_closable(true)
						.set_duration(Some(Duration::from_millis(10_000)));
				}

				deletions.push(name.clone());
			} else if name.as_str() != editor.script.name.as_str() {
				name_swaps.push((name.clone(), editor.script.name.clone()));
			}
		}

		for to_delete in deletions {
			self.scripts
				.remove(&to_delete)
				.expect("Should exist, refer to the above for loop");
		}

		for (name, new_name) in name_swaps {
			let to_swap = self
				.scripts
				.remove(&name)
				.expect("Should exist, refer to the above for loop");
			self.scripts.insert(new_name, to_swap);
		}
	}

	pub fn scripts_mut(&mut self) -> impl Iterator<Item = &mut ScriptEditor<T>> {
		self.scripts.values_mut()
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ScriptListError {}

pub struct ScriptEditor<T> {
	pub script: PocketPyScript,
	pub state: ScriptEditorState,
	marked_for_deletion: bool,
	_t: std::marker::PhantomData<T>,
}

pub enum ScriptEditorState {
	EditMode(PocketPyScript),
	DisplayMode,
}

impl ScriptEditorState {
	pub fn take_edit(&mut self) -> Option<PocketPyScript> {
		if let Self::EditMode(script) = std::mem::replace(self, ScriptEditorState::DisplayMode) {
			Some(script)
		} else {
			None
		}
	}
}

impl<T: BadgeType> ScriptEditor<T> {
	pub fn edit(&mut self) {
		self.state = ScriptEditorState::EditMode(self.script.clone());
	}

	pub fn display(&mut self) -> Result<(), PocketPyScriptError> {
		let path = T::get_path()?;

		if let Some(script) = self.state.take_edit() {
			if script.name != self.script.name {
				self.script.delete(&path)?;
			}

			self.script = script;
			self.script.save(path)?;
			self.state = ScriptEditorState::DisplayMode;
		}

		Ok(())
	}

	pub fn delete(&mut self) {
		self.marked_for_deletion = true;
	}

	pub fn widget(&mut self) -> ScriptWidget<T> {
		ScriptWidget { script: self }
	}
}
