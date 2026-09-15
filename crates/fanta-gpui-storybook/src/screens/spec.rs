//! Shared vocabulary for the state-matrix specimen stories.
//!
//! Several stories show the same component once per combination of two
//! state axes — a value state down the rows, an access level across the
//! columns — and then hand the whole thing to the reference-fixture
//! wrapper. [`NamedState`] gives those state enums one spelling for
//! "every case, and what each one is called", [`state_matrix`] lays the
//! two axes out in the existing specimen grid, and
//! [`Storybook::spec_reference`] is the one reference-fixture spelling
//! they share. Nothing here draws chrome of its own: the card, the row,
//! and the cell all come from [`super::specimen`].

use crate::*;

use super::specimen::{specimen_cell, specimen_row, specimen_rows};

/// A state enum that can name itself and enumerate its own cases.
///
/// `ALL` is the story's axis order — the reader sees the cases in the
/// order the constant lists them — and `label` is the caption printed
/// under the cell or beside the row. Stories implement this instead of
/// hand-rolling a `const SPECIMENS` array plus a `match` that names
/// them, and the shared catalog test walks `ALL` for every story rather
/// than each story repeating the same loop.
pub(crate) trait NamedState: Copy + Sized + 'static {
    /// Every case, in the order the specimen axis should show them.
    const ALL: &'static [Self];

    /// The caption the specimen grid prints for this case.
    fn label(self) -> &'static str;
}

/// A two-axis specimen grid: one titled [`specimen_row`] per row state,
/// one [`specimen_cell`] per column state inside it, stacked with the
/// shared row gap by [`specimen_rows`].
///
/// The row title is the row state's label, each cell is captioned with
/// its column state's label, and `cell` renders the specimen for that
/// pair. Pass `R::ALL` and `C::ALL` for the full matrix, or a narrower
/// slice when a story only wants part of an axis.
///
/// Each row takes its element id and debug selector from the row label,
/// so a card should mount one matrix per row enum; the enclosing
/// [`super::specimen::specimen_card`] carries the story-scoped selector.
pub(crate) fn state_matrix<R, C>(
    rows: &[R],
    columns: &[C],
    cell: impl Fn(R, C, &mut Context<Storybook>) -> AnyElement,
    cx: &mut Context<Storybook>,
) -> AnyElement
where
    R: NamedState,
    C: NamedState,
{
    let mut row_elements = Vec::with_capacity(rows.len());
    for row in rows.iter().copied() {
        let mut cells = Vec::with_capacity(columns.len());
        for column in columns.iter().copied() {
            let specimen = cell(row, column, cx);
            cells.push(specimen_cell(column.label(), None, specimen, cx));
        }
        row_elements.push(specimen_row(row.label(), row.label(), cells, cx));
    }
    specimen_rows(row_elements)
}

impl Storybook {
    /// The reference fixture the state-matrix stories mount: the story
    /// body fills the window and the intent chip appears once the story
    /// has received a real intent.
    ///
    /// The last typed intent comes from the active story's registry
    /// hook, so a story reads its own screen state without every
    /// `render_*_reference` repeating the lookup.
    pub(crate) fn spec_reference(
        &self,
        id: &'static str,
        body: AnyElement,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_reference_component_fixture(
            id,
            body,
            self.last_action_for_story(self.active_story),
            cx,
        )
    }
}
