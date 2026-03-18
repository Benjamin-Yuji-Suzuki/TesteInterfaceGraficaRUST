use chrono::{Datelike, Local, Weekday};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Clone, Copy, PartialEq)]
pub enum ModoVisao {
    Aulas,
    Monitoria,
    Intersecao,
}

#[derive(Clone)]
pub enum EstadoApp {
    Splash,
    Navegando,
    EditandoBloco,
    EditandoHorario { novo: bool },
    PerguntaIntervalo { texto_temp: String, novo: bool },
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Atividade {
    pub ativa: bool,
    pub descricao: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Horario {
    pub texto: String,
    pub is_intervalo: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Grade {
    pub horarios: Vec<Horario>,
    pub matriz: Vec<Vec<Atividade>>,
}

#[derive(Serialize, Deserialize)]
struct DadosSalvos {
    grade_aulas: Grade,
    grade_monitorias: Grade,
}

#[derive(Clone, Default)]
pub struct BlocoUI {
    pub tem_aula: bool,
    pub tem_monitoria: bool,
    pub desc_aula: String,
    pub desc_monitoria: String,
}

pub struct App {
    pub modo_atual: ModoVisao,
    pub linha_selecionada: usize,
    pub coluna_selecionada: usize,
    pub estado: EstadoApp,
    pub input_atual: String,
    
    pub grade_aulas: Grade,
    pub grade_monitorias: Grade,
    
    pub dia_atual: Weekday,
    pub hora_atual: String,
}

pub const LOGO_ASCII: &str = r#"
   _  ____ ___ _  _ ___   _    
  /_\/  __| __| \| |   \ /_\   
 / _ \ (_ | _|| .` | |) / _ \  
/_/ \_\___|___|_|\_|___/_/ \_\ 

> Pressione qualquer tecla <
"#;

impl App {
    pub fn novo_em_branco() -> Self {
        let horarios_aulas = vec![
            Horario { texto: "17:40 - 19:20".to_string(), is_intervalo: false },
            Horario { texto: "19:20 - 21:00".to_string(), is_intervalo: false },
            Horario { texto: "21:10 - 22:50".to_string(), is_intervalo: false },
        ];
        
        let horarios_monitorias = vec![
            Horario { texto: "16:30 - 17:20".to_string(), is_intervalo: false },
            Horario { texto: "17:20 - 18:10".to_string(), is_intervalo: false },
            Horario { texto: "18:10 - 19:00".to_string(), is_intervalo: false },
        ];

        let mut matriz_aulas = vec![vec![Atividade::default(); 5]; horarios_aulas.len()];
        let mut matriz_monitorias = vec![vec![Atividade::default(); 5]; horarios_monitorias.len()];

        // Colisão de teste
        matriz_monitorias[1][2] = Atividade { ativa: true, descricao: "Prog. Soft".into() };
        matriz_monitorias[2][2] = Atividade { ativa: true, descricao: "POO".into() };
        matriz_aulas[0][2]      = Atividade { ativa: true, descricao: "Redes".into() };

        let agora = Local::now();

        Self {
            modo_atual: ModoVisao::Intersecao, 
            linha_selecionada: 0,
            coluna_selecionada: 0,
            estado: EstadoApp::Splash,
            input_atual: String::new(),
            grade_aulas: Grade { horarios: horarios_aulas, matriz: matriz_aulas },
            grade_monitorias: Grade { horarios: horarios_monitorias, matriz: matriz_monitorias },
            dia_atual: agora.weekday(),
            hora_atual: agora.format("%H:%M:%S").to_string(),
        }
    }

    pub fn carregar() -> Self {
        if let Ok(conteudo) = fs::read_to_string("agenda_sync.json") {
            if let Ok(dados) = serde_json::from_str::<DadosSalvos>(&conteudo) {
                let mut app = Self::novo_em_branco();
                app.grade_aulas = dados.grade_aulas;
                app.grade_monitorias = dados.grade_monitorias;
                return app;
            }
        }
        Self::novo_em_branco()
    }

    pub fn salvar(&self) {
        let dados = DadosSalvos {
            grade_aulas: self.grade_aulas.clone(),
            grade_monitorias: self.grade_monitorias.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&dados) {
            let _ = fs::write("agenda_sync.json", json);
        }
    }

    pub fn obter_dados_visao(&self) -> (Vec<Horario>, Vec<Vec<BlocoUI>>) {
        match self.modo_atual {
            ModoVisao::Aulas => {
                let mut matriz = vec![vec![BlocoUI::default(); 5]; self.grade_aulas.horarios.len()];
                for (i, linha) in self.grade_aulas.matriz.iter().enumerate() {
                    for (j, ativ) in linha.iter().enumerate() {
                        matriz[i][j].tem_aula = ativ.ativa;
                        matriz[i][j].desc_aula = ativ.descricao.clone();
                    }
                }
                (self.grade_aulas.horarios.clone(), matriz)
            }
            ModoVisao::Monitoria => {
                let mut matriz = vec![vec![BlocoUI::default(); 5]; self.grade_monitorias.horarios.len()];
                for (i, linha) in self.grade_monitorias.matriz.iter().enumerate() {
                    for (j, ativ) in linha.iter().enumerate() {
                        matriz[i][j].tem_monitoria = ativ.ativa;
                        matriz[i][j].desc_monitoria = ativ.descricao.clone();
                    }
                }
                (self.grade_monitorias.horarios.clone(), matriz)
            }
            ModoVisao::Intersecao => self.gerar_grade_intersecao(),
        }
    }

    fn gerar_grade_intersecao(&self) -> (Vec<Horario>, Vec<Vec<BlocoUI>>) {
        let mut unicos: Vec<Horario> = Vec::new();
        
        for h in self.grade_aulas.horarios.iter().chain(self.grade_monitorias.horarios.iter()) {
            if !unicos.iter().any(|u| u.texto == h.texto) {
                unicos.push(h.clone());
            }
        }
        
        unicos.sort_by_key(|h| extrair_minutos(&h.texto).map(|(inicio, _)| inicio).unwrap_or(0));

        let mut matriz = vec![vec![BlocoUI::default(); 5]; unicos.len()];

        for (i, hor_unico) in unicos.iter().enumerate() {
            let tempo_unico = extrair_minutos(&hor_unico.texto);

            for dia in 0..5 {
                let mut bloco = BlocoUI::default();

                if let Some(t_unico) = tempo_unico {
                    // Verifica Aulas com anexo de horário
                    for (a_idx, a_hor) in self.grade_aulas.horarios.iter().enumerate() {
                        if let Some(t_aula) = extrair_minutos(&a_hor.texto) {
                            if tem_sobreposicao(t_unico, t_aula) {
                                let ativ = &self.grade_aulas.matriz[a_idx][dia];
                                if ativ.ativa {
                                    bloco.tem_aula = true;
                                    let info_aula = format!("{} [{}]", ativ.descricao, a_hor.texto);
                                    if bloco.desc_aula.is_empty() { bloco.desc_aula = info_aula; } 
                                    else { bloco.desc_aula = format!("{}\n{}", bloco.desc_aula, info_aula); }
                                }
                            }
                        }
                    }

                    // Verifica Monitorias com anexo de horário
                    for (m_idx, m_hor) in self.grade_monitorias.horarios.iter().enumerate() {
                        if let Some(t_mon) = extrair_minutos(&m_hor.texto) {
                            if tem_sobreposicao(t_unico, t_mon) {
                                let ativ = &self.grade_monitorias.matriz[m_idx][dia];
                                if ativ.ativa {
                                    bloco.tem_monitoria = true;
                                    let info_mon = format!("{} [{}]", ativ.descricao, m_hor.texto);
                                    if bloco.desc_monitoria.is_empty() { bloco.desc_monitoria = info_mon; } 
                                    else { bloco.desc_monitoria = format!("{}\n{}", bloco.desc_monitoria, info_mon); }
                                }
                            }
                        }
                    }
                }
                matriz[i][dia] = bloco;
            }
        }

        (unicos, matriz)
    }

    pub fn atualizar_relogio(&mut self) {
        let agora = Local::now();
        self.hora_atual = agora.format("%H:%M:%S").to_string();
        self.dia_atual = agora.weekday();
    }

    pub fn traduzir_dia(&self) -> &'static str {
        match self.dia_atual {
            Weekday::Mon => "Segunda", Weekday::Tue => "Terça", Weekday::Wed => "Quarta",
            Weekday::Thu => "Quinta", Weekday::Fri => "Sexta", Weekday::Sat => "Sábado", Weekday::Sun => "Domingo",
        }
    }
}

fn extrair_minutos(texto: &str) -> Option<(u32, u32)> {
    let partes: Vec<&str> = texto.split('-').collect();
    if partes.len() != 2 { return None; }
    
    let p1: Vec<&str> = partes[0].trim().split(':').collect();
    let p2: Vec<&str> = partes[1].trim().split(':').collect();
    if p1.len() != 2 || p2.len() != 2 { return None; }
    
    let h1: u32 = p1[0].parse().ok()?;
    let m1: u32 = p1[1].parse().ok()?;
    let h2: u32 = p2[0].parse().ok()?;
    let m2: u32 = p2[1].parse().ok()?;
    
    Some((h1 * 60 + m1, h2 * 60 + m2))
}

fn tem_sobreposicao(a: (u32, u32), b: (u32, u32)) -> bool {
    a.0 < b.1 && b.0 < a.1
}