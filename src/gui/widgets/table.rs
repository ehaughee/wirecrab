use iced::{
    Element, Length,
    widget::{column, container, row, text},
};
use std::marker::PhantomData;

/// Create a scrollable table widget for Iced
pub struct IcedTable<Message> {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<Length>,
    _phantom: PhantomData<Message>,
}

impl<Message> IcedTable<Message> {
    pub fn new(headers: Vec<String>) -> Self {
        let column_count = headers.len();
        let column_widths = vec![Length::Fill; column_count];

        Self {
            headers,
            rows: Vec::new(),
            column_widths,
            _phantom: PhantomData,
        }
    }

    pub fn with_column_widths(mut self, widths: Vec<Length>) -> Self {
        self.column_widths = widths;
        self
    }

    pub fn add_row(mut self, row: Vec<String>) -> Self {
        self.rows.push(row);
        self
    }

    pub fn with_rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }
}

impl<Message> IcedTable<Message>
where
    Message: 'static,
{
    pub fn view(self) -> Element<'static, Message> {
        let mut table_column = column![].spacing(2);

        // Create header row
        let mut header_row = row![].spacing(10).padding(10);
        for (i, header) in self.headers.iter().enumerate() {
            let width = self.column_widths.get(i).unwrap_or(&Length::Fill);
            header_row = header_row.push(text(header.clone()).size(16).width(*width));
        }

        // Style the header
        let styled_header = container(header_row).padding(5);

        table_column = table_column.push(styled_header);

        // Create data rows
        for row_data in &self.rows {
            let mut data_row = row![].spacing(10).padding(5);

            for (i, cell) in row_data.iter().enumerate() {
                let width = self.column_widths.get(i).unwrap_or(&Length::Fill);
                data_row = data_row.push(text(cell.clone()).size(14).width(*width));
            }

            // Style the data row
            let styled_row = container(data_row).padding(2);

            table_column = table_column.push(styled_row);
        }

        table_column.into()
    }
}

/// Convenience function to create a simple string table
pub fn create_table<Message: 'static>(
    headers: Vec<String>,
    data: Vec<Vec<String>>,
    column_widths: Option<Vec<Length>>,
) -> Element<'static, Message> {
    let mut table = IcedTable::new(headers).with_rows(data);

    if let Some(widths) = column_widths {
        table = table.with_column_widths(widths);
    }

    table.view()
}
