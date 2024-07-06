use std::alloc::{handle_alloc_error, Layout, LayoutError};
use std::fmt::{Display, Error as FmtError, Formatter};

use TryReserveErrorKind::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TryReserveError {
    kind: TryReserveErrorKind,
}

impl TryReserveError {
    #[must_use]
    pub fn kind(&self) -> TryReserveErrorKind {
        self.kind.clone()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TryReserveErrorKind {
    CapacityOverflow,
    AllocError { layout: Layout },
}

pub(crate) fn handle_reserve<T>(result: Result<T, TryReserveError>) -> T {
    match result.map_err(|e| e.kind()) {
        Ok(res) => res,
        Err(CapacityOverflow) => capacity_overflow(),
        Err(AllocError { layout }) => handle_alloc_error(layout),
    }
}

fn capacity_overflow() -> ! {
    panic!("capacity overflow");
}

impl From<TryReserveErrorKind> for TryReserveError {
    fn from(kind: TryReserveErrorKind) -> Self {
        Self { kind }
    }
}

impl From<LayoutError> for TryReserveErrorKind {
    fn from(_: LayoutError) -> Self {
        CapacityOverflow
    }
}

impl Display for TryReserveError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), FmtError> {
        fmt.write_str("memory allocation failed")?;
        let reason = match self.kind {
            CapacityOverflow => " because the computed capacity exceeded the collection's maximum",
            AllocError { .. } => " because the memory allocator returned an error",
        };
        fmt.write_str(reason)
    }
}
