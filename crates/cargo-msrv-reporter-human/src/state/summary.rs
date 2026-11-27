use cargo_msrv_context::SelectedPackage;
use msrv::MSRV;

pub struct Summary {
    rows: Vec<SummaryRow>,
}

pub struct SummaryRow {
    pub package: SelectedPackage,
    pub outcome: Outcome,
    pub written: bool,
}

pub enum Outcome {
    Msrv(Option<MSRV>),
    Verified {
        version: semver::Version,
        compatible: bool,
    },
    Version(MSRV),
}

impl Summary {
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    pub fn msrv(&mut self, package: Option<&SelectedPackage>, msrv: Option<&MSRV>) {
        self.push(package, Outcome::Msrv(msrv.cloned()));
    }

    pub fn verified(
        &mut self,
        package: Option<&SelectedPackage>,
        version: &semver::Version,
        compatible: bool,
    ) {
        let outcome = Outcome::Verified {
            version: version.clone(),
            compatible,
        };

        self.push(package, outcome);
    }

    pub fn version(&mut self, package: Option<&SelectedPackage>, version: &MSRV) {
        self.push(package, Outcome::Version(version.clone()));
    }

    pub fn written(&mut self, package: Option<&SelectedPackage>, version: &MSRV) {
        let Some(package) = package else {
            return;
        };

        match self.rows.iter_mut().find(|row| &row.package == package) {
            Some(row) => row.written = true,
            None => self.push(Some(package), Outcome::Version(version.clone())),
        }
    }

    pub fn is_shown(&self) -> bool {
        self.rows.len() > 1
    }

    pub fn rows(&self) -> &[SummaryRow] {
        &self.rows
    }

    fn push(&mut self, package: Option<&SelectedPackage>, outcome: Outcome) {
        if let Some(package) = package {
            self.rows.push(SummaryRow {
                package: package.clone(),
                outcome,
                written: false,
            });
        }
    }
}
