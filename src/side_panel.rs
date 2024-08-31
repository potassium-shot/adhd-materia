#[derive(Default, kinded::Kinded)]
pub enum SidePanel {
	#[default]
	Hidden,
	ScheduledTasks {},
	Tags {},
}

impl SidePanel {
	pub fn show(&mut self, ui: &mut egui::Ui) {
		match self {
			Self::Hidden => {}
			Self::ScheduledTasks {} => {
				ui.label("Would you rather have unlimited bacon, but no games, OR, games, UNLIMITED games, but no games?");
			}
			Self::Tags {} => {
				ui.label("Dinosaur haha");
			}
		}
	}

	pub fn toggle(&mut self, kind: SidePanelKind) {
		if self.kind() == kind {
			self.hide();
		} else {
			self.open(kind);
		}
	}

	pub fn open(&mut self, kind: SidePanelKind) {
		*self = match kind {
			SidePanelKind::ScheduledTasks => Self::ScheduledTasks {},
			SidePanelKind::Tags => Self::Tags {},
			SidePanelKind::Hidden => Self::Hidden,
		}
	}

	pub fn hide(&mut self) {
		*self = Self::Hidden;
	}

	pub fn is_shown(&self) -> bool {
		match self {
			Self::Hidden => false,
			_ => true,
		}
	}
}
