mod app;
mod ui;

use std::{io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
// Repare que importamos o Stdout aqui
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, Bloco, EstadoApp, Horario, ModoVisao};
use ui::desenhar_interface;

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Lendo argumentos do terminal para o recurso de CLI (--reset)
    let args: Vec<String> = std::env::args().collect();
    
    // Instanciando corretamente usando as novas funções
    let mut app = if args.contains(&String::from("--reset")) {
        App::novo_em_branco()
    } else {
        App::carregar()
    };

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

// Assinatura simplificada: dizemos exatamente que é o Crossterm com Stdout
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        app.atualizar_relogio();

        terminal.draw(|f| desenhar_interface(f, app))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                let estado_clone = app.estado.clone();
                
                match estado_clone {
                    EstadoApp::Navegando => {
                        match key.code {
                            // CORREÇÃO DO BLOCO MATCH AQUI: uso de { }
                            KeyCode::Char('q') => {
                                app.salvar();
                                return Ok(());
                            }
                            KeyCode::Char('1') => app.modo_atual = ModoVisao::Aulas,
                            KeyCode::Char('2') => app.modo_atual = ModoVisao::Monitoria,
                            KeyCode::Char('3') => app.modo_atual = ModoVisao::Intersecao,
                            
                            KeyCode::Up => app.linha_selecionada = app.linha_selecionada.saturating_sub(1),
                            KeyCode::Down => app.linha_selecionada = (app.linha_selecionada + 1).min(app.horarios.len().saturating_sub(1)),
                            KeyCode::Left => app.coluna_selecionada = app.coluna_selecionada.saturating_sub(1),
                            KeyCode::Right => app.coluna_selecionada = (app.coluna_selecionada + 1).min(4),
                            
                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                app.estado = EstadoApp::EditandoHorario { novo: true };
                                app.input_atual = String::new();
                            }
                            KeyCode::Char('m') | KeyCode::Char('M') => {
                                if !app.horarios.is_empty() {
                                    app.estado = EstadoApp::EditandoHorario { novo: false };
                                    app.input_atual = app.horarios[app.linha_selecionada].texto.clone();
                                }
                            }
                            
                            KeyCode::Enter => {
                                if app.modo_atual != ModoVisao::Intersecao && !app.horarios.is_empty() {
                                    if !app.horarios[app.linha_selecionada].is_intervalo {
                                        app.estado = EstadoApp::EditandoBloco;
                                        app.input_atual = String::new();
                                    }
                                }
                            }
                            KeyCode::Delete => {
                                if !app.horarios.is_empty() && !app.horarios[app.linha_selecionada].is_intervalo {
                                    let bloco = &mut app.matriz[app.linha_selecionada][app.coluna_selecionada];
                                    if app.modo_atual == ModoVisao::Aulas {
                                        bloco.desc_aula.clear();
                                        bloco.tem_aula = false;
                                    } else if app.modo_atual == ModoVisao::Monitoria {
                                        bloco.desc_monitoria.clear();
                                        bloco.tem_monitoria = false;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    
                    EstadoApp::EditandoBloco => {
                        match key.code {
                            KeyCode::Enter => {
                                let bloco = &mut app.matriz[app.linha_selecionada][app.coluna_selecionada];
                                if app.modo_atual == ModoVisao::Aulas {
                                    bloco.desc_aula = app.input_atual.clone();
                                    bloco.tem_aula = !bloco.desc_aula.is_empty();
                                } else if app.modo_atual == ModoVisao::Monitoria {
                                    bloco.desc_monitoria = app.input_atual.clone();
                                    bloco.tem_monitoria = !bloco.desc_monitoria.is_empty();
                                }
                                app.estado = EstadoApp::Navegando;
                            }
                            KeyCode::Esc => app.estado = EstadoApp::Navegando,
                            KeyCode::Backspace => { app.input_atual.pop(); }
                            KeyCode::Char(c) => app.input_atual.push(c),
                            _ => {}
                        }
                    }

                    EstadoApp::EditandoHorario { novo } => {
                        match key.code {
                            KeyCode::Enter => {
                                app.estado = EstadoApp::PerguntaIntervalo { texto_temp: app.input_atual.clone(), novo };
                            }
                            KeyCode::Esc => app.estado = EstadoApp::Navegando,
                            KeyCode::Backspace => { app.input_atual.pop(); }
                            KeyCode::Char(c) => app.input_atual.push(c),
                            _ => {}
                        }
                    }

                    EstadoApp::PerguntaIntervalo { texto_temp, novo } => {
                        match key.code {
                            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Char('n') | KeyCode::Char('N') => {
                                let is_intervalo = matches!(key.code, KeyCode::Char('s') | KeyCode::Char('S'));
                                
                                if novo {
                                    app.horarios.push(Horario { texto: texto_temp, is_intervalo });
                                    app.matriz.push(vec![Bloco::default(); 5]);
                                } else {
                                    app.horarios[app.linha_selecionada] = Horario { texto: texto_temp, is_intervalo };
                                    if is_intervalo {
                                        for dia_idx in 0..5 {
                                            app.matriz[app.linha_selecionada][dia_idx] = Bloco::default();
                                        }
                                    }
                                }
                                app.estado = EstadoApp::Navegando;
                            }
                            KeyCode::Esc => app.estado = EstadoApp::Navegando,
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}