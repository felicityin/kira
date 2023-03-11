use super::func::FunctionInfo;

pub struct Scopes {
  pub cur_func: Option<FunctionInfo>,
}

/// Returns a reference to the current function information.
macro_rules! cur_func {
    ($scopes:expr) => {
        $scopes.cur_func.as_ref().unwrap()
    };
}
pub(crate) use cur_func;

/// Returns a mutable reference to the current function information.
macro_rules! cur_func_mut {
    ($scopes:expr) => {
        $scopes.cur_func.as_mut().unwrap()
    };
}
pub(crate) use cur_func_mut;

impl Scopes {
    /// Creates a new `Scopes`.
    pub fn new() -> Self {
        Self {
            cur_func: None,
        }
    }

    /// Returns `true` if is currently in global scope.
    pub fn _is_global(&self) -> bool {
        self.cur_func.is_none()
    }
}
