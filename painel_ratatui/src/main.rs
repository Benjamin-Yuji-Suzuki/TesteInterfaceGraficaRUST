mod app;
mod ui;

use std::{io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode}, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use app::{App, Atividade, EstadoApp, Horario, ModoVisao};
use ui::desenhar_interface;

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let args: Vec<String> = std::env::args().collect();
    let mut app = if args.contains(&String::from("--reset")) { App::novo_em_branco() } else { App::carregar() };

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res { println!("{:?}", err) }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        app.atualizar_relogio();
        terminal.draw(|f| desenhar_interface(f, app))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                let estado_clone = app.estado.clone();
                
                match estado_clone {
                    EstadoApp::Splash => { app.estado = EstadoApp::Navegando; }

                    EstadoApp::Navegando => {
                        match key.code {
                            KeyCode::Char('q') => { app.salvar(); return Ok(()); }
                            
                            // Ao mudar de aba, zera os cursores
                            KeyCode::Char('1') => { app.modo_atual = ModoVisao::Aulas; app.linha_selecionada = 0; app.coluna_selecionada = 0; }
                            KeyCode::Char('2') => { app.modo_atual = ModoVisao::Monitoria; app.linha_selecionada = 0; app.coluna_selecionada = 0; }
                            KeyCode::Char('3') => { app.modo_atual = ModoVisao::Intersecao; app.linha_selecionada = 0; app.coluna_selecionada = 0; }
                            
                            KeyCode::Up => app.linha_selecionada = app.linha_selecionada.saturating_sub(1),
                            KeyCode::Down => {
                                let max = app.obter_dados_visao().0.len().saturating_sub(1);
                                app.linha_selecionada = (app.linha_selecionada + 1).min(max);
                            }
                            KeyCode::Left => app.coluna_selecionada = app.coluna_selecionada.saturating_sub(1),
                            KeyCode::Right => app.coluna_selecionada = (app.coluna_selecionada + 1).min(4),
                            
                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                if app.modo_atual != ModoVisao::Intersecao {
                                    app.estado = EstadoApp::EditandoHorario { novo: true };
                                    app.input_atual = String::new();
                                }
                            }
                            KeyCode::Char('m') | KeyCode::Char('M') => {
                                if app.modo_atual != ModoVisao::Intersecao {
                                    let grade = if app.modo_atual == ModoVisao::Aulas { &app.grade_aulas } else { &app.grade_monitorias };
                                    if !grade.horarios.is_empty() {
                                        let texto = grade.horarios[app.linha_selecionada].texto.clone();
                                        app.estado = EstadoApp::EditandoHorario { novo: false };
                                        app.input_atual = texto;
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                if app.modo_atual != ModoVisao::Intersecao {
                                    let grade = if app.modo_atual == ModoVisao::Aulas { &app.grade_aulas } else { &app.grade_monitorias };
                                    if !grade.horarios.is_empty() && !grade.horarios[app.linha_selecionada].is_intervalo {
                                        app.estado = EstadoApp::EditandoBloco;
                                        app.input_atual = String::new();
                                    }
                                }
                            }
                            KeyCode::Delete => {
                                if app.modo_atual != ModoVisao::Intersecao {
                                    // 1. Extraímos os valores em variáveis locais primeiro
                                    let l = app.linha_selecionada;
                                    let c = app.coluna_selecionada;
                                    
                                    // 2. Só então pedimos acesso de modificação
                                    let grade = if app.modo_atual == ModoVisao::Aulas { &mut app.grade_aulas } else { &mut app.grade_monitorias };
                                    
                                    if !grade.horarios.is_empty() && !grade.horarios[l].is_intervalo {
                                        grade.matriz[l][c] = Atividade::default();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    
                    EstadoApp::EditandoBloco => {
                        match key.code {
                            KeyCode::Enter => {
                                if app.modo_atual != ModoVisao::Intersecao {
                                    let l = app.linha_selecionada;
                                    let c = app.coluna_selecionada;
                                    let input = app.input_atual.clone();
                                    
                                    let grade = if app.modo_atual == ModoVisao::Aulas { &mut app.grade_aulas } else { &mut app.grade_monitorias };
                                    
                                    grade.matriz[l][c] = Atividade {
                                        ativa: !input.is_empty(),
                                        descricao: input,
                                    };
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
                            KeyCode::Enter => { app.estado = EstadoApp::PerguntaIntervalo { texto_temp: app.input_atual.clone(), novo }; }
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
                                
                                if app.modo_atual != ModoVisao::Intersecao {
                                    let l = app.linha_selecionada;
                                    let grade = if app.modo_atual == ModoVisao::Aulas { &mut app.grade_aulas } else { &mut app.grade_monitorias };
                                    
                                    if novo {
                                        grade.horarios.push(Horario { texto: texto_temp, is_intervalo });
                                        grade.matriz.push(vec![Atividade::default(); 5]);
                                    } else {
                                        grade.horarios[l] = Horario { texto: texto_temp, is_intervalo };
                                        if is_intervalo {
                                            grade.matriz[l] = vec![Atividade::default(); 5];
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