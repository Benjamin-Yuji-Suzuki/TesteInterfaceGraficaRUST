use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs},
};
use crate::app::{App, BlocoUI, EstadoApp, ModoVisao};

fn area_centralizada(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)])
        .split(popup_layout[1])[1]
}

// O motor de renderização da célula com ícones e múltiplas linhas
fn calcular_celula<'a>(bloco: &'a BlocoUI, modo: ModoVisao) -> Text<'a> {
    match modo {
        ModoVisao::Aulas => {
            if bloco.tem_aula { Text::from(Span::styled(bloco.desc_aula.clone(), Style::default().fg(Color::Blue))) } 
            else { Text::from(Span::styled("-", Style::default().fg(Color::DarkGray))) }
        }
        ModoVisao::Monitoria => {
            if bloco.tem_monitoria { Text::from(Span::styled(bloco.desc_monitoria.clone(), Style::default().fg(Color::Green))) } 
            else { Text::from(Span::styled("-", Style::default().fg(Color::DarkGray))) }
        }
        ModoVisao::Intersecao => {
            if bloco.tem_aula && bloco.tem_monitoria {
                let mut lines = vec![];
                lines.push(Line::from(vec![Span::styled("⚠️ CONFLITO", Style::default().fg(Color::Rgb(255, 85, 0)).add_modifier(Modifier::BOLD))]));
                
                for linha in bloco.desc_aula.lines() {
                    lines.push(Line::from(vec![Span::styled(format!("📚 {}", linha), Style::default().fg(Color::Blue))]));
                }
                for linha in bloco.desc_monitoria.lines() {
                    lines.push(Line::from(vec![Span::styled(format!("👨‍🏫 {}", linha), Style::default().fg(Color::Green))]));
                }
                Text::from(lines)
            } else if bloco.tem_aula {
                let lines: Vec<Line> = bloco.desc_aula.lines().map(|l| Line::from(vec![Span::styled(format!("📚 {}", l), Style::default().fg(Color::Blue))])).collect();
                Text::from(lines)
            } else if bloco.tem_monitoria {
                let lines: Vec<Line> = bloco.desc_monitoria.lines().map(|l| Line::from(vec![Span::styled(format!("👨‍🏫 {}", l), Style::default().fg(Color::Green))])).collect();
                Text::from(lines)
            } else { 
                Text::from(Span::styled("Livre", Style::default().fg(Color::DarkGray))) 
            }
        }
    }
}

fn desenhar_splash(f: &mut Frame, area: Rect) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)])
        .split(area);
        
    // Verifica a largura da tela. Se for menor que 80 colunas, usa um texto simples.
    let conteudo = if area.width < 80 {
        "AGENDA SYNC\n\n> Pressione qualquer tecla para iniciar <"
    } else {
        crate::app::LOGO_ASCII
    };

    let splash = Paragraph::new(conteudo)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center).block(Block::default().borders(Borders::NONE));
        
    f.render_widget(splash, chunks[1]);
}

pub fn desenhar_interface(frame: &mut Frame, app: &App) {
    let area = frame.size();
    if let EstadoApp::Splash = app.estado { desenhar_splash(frame, area); return; }

    let layout = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let texto_cabecalho = format!(" 📅 Hoje é {} - ⏰ {}", app.traduzir_dia(), app.hora_atual);
    let cabecalho = Paragraph::new(texto_cabecalho).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title(" Agenda Sync "));
    frame.render_widget(cabecalho, layout[0]);

    let titulos_abas: Vec<Line> = vec!["1. Aulas (Faculdade)", "2. Monitorias", "3. Interseção Geral"]
        .into_iter().map(Line::from).collect();
    
    let aba_selecionada = match app.modo_atual { ModoVisao::Aulas => 0, ModoVisao::Monitoria => 1, ModoVisao::Intersecao => 2 };

    let abas = Tabs::new(titulos_abas).block(Block::default().borders(Borders::ALL).title(" Visão ")).select(aba_selecionada)
        .style(Style::default().fg(Color::DarkGray)).highlight_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)).divider(" | ");
    frame.render_widget(abas, layout[1]);

    let (horarios, matriz) = app.obter_dados_visao();
    let mut linhas_tabela = Vec::new();

    for (i, horario) in horarios.iter().enumerate() {
        let mut estilo_hora = Style::default().fg(if horario.is_intervalo { Color::Yellow } else { Color::DarkGray });
        if horario.is_intervalo { estilo_hora = estilo_hora.add_modifier(Modifier::BOLD); }
        
        let linha_hora = Line::from(horario.texto.clone()).alignment(Alignment::Center);
        let mut celulas = vec![Cell::from(linha_hora).style(estilo_hora)];
        
        for dia_idx in 0..5 {
            let mut celula;
            if horario.is_intervalo {
                let text = Text::from(Line::from("INTERVALO").alignment(Alignment::Center));
                celula = Cell::from(text).style(Style::default().fg(Color::Yellow).bg(Color::Black).add_modifier(Modifier::BOLD));
            } else {
                let bloco = &matriz[i][dia_idx];
                let mut text = calcular_celula(bloco, app.modo_atual);
                for line in &mut text.lines { line.alignment = Some(Alignment::Center); }
                
                let mut estilo_fundo = Style::default();
                if dia_idx as u32 == app.dia_atual.num_days_from_monday() { estilo_fundo = estilo_fundo.bg(Color::Rgb(30, 30, 30)); }

                if app.modo_atual != ModoVisao::Intersecao && matches!(app.estado, EstadoApp::Navegando | EstadoApp::EditandoBloco) {
                    if i == app.linha_selecionada && dia_idx == app.coluna_selecionada {
                        estilo_fundo = estilo_fundo.bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD);
                    }
                }
                celula = Cell::from(text).style(estilo_fundo);
            }

            if horario.is_intervalo && app.modo_atual != ModoVisao::Intersecao && matches!(app.estado, EstadoApp::Navegando | EstadoApp::EditandoBloco) {
                if i == app.linha_selecionada && dia_idx == app.coluna_selecionada {
                    celula = celula.style(Style::default().fg(Color::Yellow).bg(Color::DarkGray).add_modifier(Modifier::BOLD));
                }
            }
            celulas.push(celula);
        }
        
        // Ajuste dinâmico da altura da linha para caber múltiplas informações na Interseção
        let altura_linha = if app.modo_atual == ModoVisao::Intersecao { 4 } else { 2 };
        linhas_tabela.push(Row::new(celulas).height(altura_linha));
    }

    let larguras = [
        Constraint::Length(15), Constraint::Percentage(17), Constraint::Percentage(17), 
        Constraint::Percentage(17), Constraint::Percentage(17), Constraint::Percentage(17), 
    ];

    let header_row = Row::new(vec![
        Cell::from(Line::from("Horário").alignment(Alignment::Center)),
        Cell::from(Line::from("Segunda").alignment(Alignment::Center)),
        Cell::from(Line::from("Terça").alignment(Alignment::Center)),
        Cell::from(Line::from("Quarta").alignment(Alignment::Center)),
        Cell::from(Line::from("Quinta").alignment(Alignment::Center)),
        Cell::from(Line::from("Sexta").alignment(Alignment::Center)),
    ]).style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)).bottom_margin(1);

    let tabela = Table::new(linhas_tabela, larguras).block(Block::default().borders(Borders::ALL).title(" Grade Horária ")).header(header_row);
    frame.render_widget(tabela, layout[2]);

    let texto_rodape = match app.estado {
        EstadoApp::Navegando => if app.modo_atual == ModoVisao::Intersecao { " [1/2] Alternar Grade | [Q] Sair (Modo Leitura)" } 
                                else { " [Setas] Navegar | [Enter] Editar | [A] Novo Horário | [M] Modificar Horário | [Q] Sair" },
        EstadoApp::EditandoBloco | EstadoApp::EditandoHorario { .. } => " [Enter] Salvar | [Esc] Cancelar | [Backspace] Apagar",
        EstadoApp::PerguntaIntervalo { .. } => " [S] Sim | [N] Não",
        _ => ""
    };
    frame.render_widget(Paragraph::new(texto_rodape).style(Style::default().fg(Color::DarkGray)), layout[3]);

    match &app.estado {
        EstadoApp::EditandoBloco => {
            let area_popup = area_centralizada(70, 20, area); frame.render_widget(Clear, area_popup); 
            let titulo = if app.modo_atual == ModoVisao::Aulas { " Editando Aula " } else { " Editando Monitoria " };
            let input = Paragraph::new(format!("{}_", app.input_atual)).style(Style::default().fg(Color::Green)).block(Block::default().borders(Borders::ALL).title(titulo));
            frame.render_widget(input, area_popup);
        }
        EstadoApp::EditandoHorario { novo } => {
            let area_popup = area_centralizada(50, 20, area); frame.render_widget(Clear, area_popup); 
            let titulo = if *novo { " Adicionar Horário " } else { " Modificar Horário " };
            let input = Paragraph::new(format!("{}_", app.input_atual)).style(Style::default().fg(Color::Cyan)).block(Block::default().borders(Borders::ALL).title(titulo));
            frame.render_widget(input, area_popup);
        }
        EstadoApp::PerguntaIntervalo { texto_temp, .. } => {
            let area_popup = area_centralizada(40, 20, area); frame.render_widget(Clear, area_popup); 
            let pergunta = Paragraph::new(format!("O horário '{}' é um intervalo/pausa?\n\n[S] Sim ou [N] Não.", texto_temp))
                .style(Style::default().fg(Color::Yellow)).block(Block::default().borders(Borders::ALL).title(" Configuração "));
            frame.render_widget(pergunta, area_popup);
        }
        _ => {}
    }
}