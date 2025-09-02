// use masonry::properties::types::Length;
use xilem::view::{GridExt, SizedBox, grid, label, sized_box};
use xilem::{WidgetView, palette};

const GRID_GAP: f64 = 2.;

/// Simple string table function that creates a properly aligned table
///
/// Creates a table where both headers and data are strings, rendered as labels.
/// This is a convenience function for the common case of displaying string data.
///
/// # Arguments
/// * `headers` - Column header labels
/// * `data` - 2D array of string data where data[row][col] contains the cell content
pub fn string_table<S: 'static>(headers: Vec<&str>, data: Vec<Vec<String>>) -> impl WidgetView<S> {
    // Create all the view items for the grid
    let mut table_items: Vec<xilem::view::GridItem<SizedBox<xilem::view::Label, S>, S, ()>> =
        Vec::new();

    // Build Label GridItems at the right positions for the table headers
    for (col_index, header) in headers.iter().enumerate() {
        table_items.push(
            sized_box(label(*header))
                .height(25.)
                .padding(2.)
                .border(palette::css::RED, 1.)
                .grid_pos(col_index as i32, 0),
        );
    }

    // Build Label GridItems for the data rows
    for (row_index, row) in data.iter().enumerate() {
        for (col_index, cell) in row.iter().enumerate() {
            table_items.push(
                sized_box(label(cell.as_str()))
                    .height(25.)
                    .padding(2.)
                    .grid_pos(col_index as i32, (row_index as i32) + 1),
            );
        }
    }

    return grid(table_items, headers.len() as i32, (data.len() + 1) as i32).spacing(GRID_GAP);
}

// pub fn table_header<S: 'static>(cols: &[&str]) -> impl WidgetView<S> + use<S> {
//     let cells = cols.iter().map(|c| label(*c)).collect::<Vec<_>>();

//     flex(cells).direction(Axis::Horizontal)
// }

// pub fn table_rows<S: 'static, R: WidgetView<S> + 'static>(
//     rows: Vec<R>,
// ) -> impl WidgetView<S> + use<S, R> {
//     flex(rows).direction(Axis::Vertical)
// }

// pub fn table_row<S: 'static, C: WidgetView<S> + 'static>(
//     cells: Vec<C>,
// ) -> impl WidgetView<S> + use<S, C> {
//     flex(cells).direction(Axis::Horizontal)
// }
