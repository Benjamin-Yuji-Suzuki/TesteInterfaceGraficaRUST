use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::io::{stdout, Result};

fn main() -> Result<()> {
    // Setup: Entra na tela alternativa e desativa o eco de teclas
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Game Loop
    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            
            let texto = Paragraph::new("Hello World, Nível 2!\n\nAperte 'q' para sair.")
                .block(Block::default().title(" Meu Painel Ratatui ").borders(Borders::ALL))
                .centered();
            
            frame.render_widget(texto, area);
        })?;

        // Escuta o teclado
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Teardown: Limpa a tela e devolve o terminal ao normal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}