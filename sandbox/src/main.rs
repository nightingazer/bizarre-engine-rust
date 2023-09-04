use bizarre_engine::core::logger::{
    terminal_escape_code::{bg_color, BOLD, RED, RESET},
    terminal_macros::*,
};

fn main() {
    {
        let esc_sequence = escape_sequence!(BOLD, RED);
        println!("{}Hello world!", esc_sequence);
    }
    {
        let esc_sequence = escape_sequence!(RESET, RED);
        println!("{}Hello world!", esc_sequence);
    }
    {
        let esc_sequence = escape_sequence!(RESET, bg_color(RED));
        println!("{}Hello world!", esc_sequence);
    }
    let esc_sequence = escape_sequence!(RESET);
    println!("{}Hello world!", esc_sequence);
}
