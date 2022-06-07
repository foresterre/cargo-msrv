use crate::ReleaseSource;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Action {
    status: ActionStatus,
    details: ActionDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope_position: Option<ScopePosition>,
}

impl Action {
    fn new(action: ActionDetails) -> Self {
        Self {
            status: (&action).into(),
            details: action,
            scope_position: None,
        }
    }

    pub(in crate::storyteller) fn clone_with_scope_position(
        &self,
        position: ScopePosition,
    ) -> Self {
        let mut cloned = self.clone();
        cloned.scope_position = Some(position);
        cloned
    }

    pub fn status(&self) -> ActionStatus {
        self.status
    }

    pub fn details(&self) -> &ActionDetails {
        &self.details
    }
}

/// Specialized `new` methods which provide a shortcut to create actions.
///
/// Without these short cuts, you would have to create an action like so (assuming `new` would
/// be public):
///
/// ```no_run
/// Action::new(ActionDetails::FetchingIndex { source });
/// ```
impl Action {
    pub fn fetching_index(source: ReleaseSource) -> Self {
        Self::new(ActionDetails::FetchingIndex { source })
    }
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionDetails {
    FetchingIndex { source: ReleaseSource },
}

impl<'reference> From<&'reference ActionDetails> for ActionStatus {
    fn from(action_details: &'reference ActionDetails) -> Self {
        match action_details {
            ActionDetails::FetchingIndex { .. } => Self::Fetching,
        }
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Fetching,
    Downloading,
    Checking,
}

impl ActionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fetching => "Fetching",
            Self::Downloading => "Downloading",
            Self::Checking => "Checking",
        }
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize)]
pub enum ScopePosition {
    Begin,
    End,
}
