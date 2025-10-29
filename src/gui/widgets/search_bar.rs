use iced::{Element, widget::text_input};

pub fn search_bar<'a, Message>(
    placeholder: &'a str,
    value: &'a str,
    on_input: fn(String) -> Message,
) -> Element<'a, Message>
where
    Message: Clone + 'static,
{
    text_input(placeholder, value)
        .on_input(on_input)
        .padding(10)
        .size(16)
        .into()
}