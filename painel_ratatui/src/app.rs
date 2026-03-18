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
    Navegando,
    EditandoBloco,
    EditandoHorario { novo: bool },
    PerguntaIntervalo { texto_temp: String, novo: bool },
}

// Avisamos ao serde que essas structs podem virar JSON
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Bloco {
    pub tem_aula: bool,
    pub tem_monitoria: bool,
    pub desc_aula: String,
    pub desc_monitoria: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Horario {
    pub texto: String,
    pub is_intervalo: bool,
}

// Estrutura auxiliar apenas para o arquivo JSON (salvamos apenas os dados, não a interface)
#[derive(Serialize, Deserialize)]
struct DadosSalvos {
    horarios: Vec<Horario>,
    matriz: Vec<Vec<Bloco>>,
}

pub struct App {
    pub modo_atual: ModoVisao,
    pub linha_selecionada: usize,
    pub coluna_selecionada: usize,
    pub estado: EstadoApp,
    pub input_atual: String,
    pub horarios: Vec<Horario>,
    pub matriz: Vec<Vec<Bloco>>,
    pub dia_atual: Weekday,
    pub hora_atual: String,
}

impl App {
    // 1. O Template Limpo para o GitHub
    pub fn novo_em_branco() -> Self {
        let horarios_iniciais = vec![
            Horario { texto: "14:00 - 15:40".to_string(), is_intervalo: false },
            Horario { texto: "15:40 - 16:00".to_string(), is_intervalo: true }, 
            Horario { texto: "16:00 - 17:40".to_string(), is_intervalo: false },
            Horario { texto: "17:40 - 19:00".to_string(), is_intervalo: true }, 
            Horario { texto: "19:00 - 20:40".to_string(), is_intervalo: false },
            Horario { texto: "20:50 - 22:30".to_string(), is_intervalo: false },
        ];
        
        let qtd = horarios_iniciais.len();
        // Cria a matriz completamente vazia
        let matriz = vec![vec![Bloco::default(); 5]; qtd];

        let agora = Local::now();

        Self {
            modo_atual: ModoVisao::Intersecao, 
            linha_selecionada: 0,
            coluna_selecionada: 0,
            estado: EstadoApp::Navegando,
            input_atual: String::new(),
            horarios: horarios_iniciais,
            matriz,
            dia_atual: agora.weekday(),
            hora_atual: agora.format("%H:%M:%S").to_string(),
        }
    }

    // 2. Carregar do HD (Se não existir, cria um em branco)
    pub fn carregar() -> Self {
        if let Ok(conteudo) = fs::read_to_string("agenda_sync.json") {
            if let Ok(dados) = serde_json::from_str::<DadosSalvos>(&conteudo) {
                let mut app = Self::novo_em_branco();
                app.horarios = dados.horarios;
                app.matriz = dados.matriz;
                return app;
            }
        }
        Self::novo_em_branco()
    }

    // 3. Salvar no HD
    pub fn salvar(&self) {
        let dados = DadosSalvos {
            horarios: self.horarios.clone(),
            matriz: self.matriz.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&dados) {
            let _ = fs::write("agenda_sync.json", json);
        }
    }

    pub fn atualizar_relogio(&mut self) {
        let agora = Local::now();
        self.hora_atual = agora.format("%H:%M:%S").to_string();
        self.dia_atual = agora.weekday();
    }

    pub fn traduzir_dia(&self) -> &'static str {
        match self.dia_atual {
            Weekday::Mon => "Segunda",
            Weekday::Tue => "Terça",
            Weekday::Wed => "Quarta",
            Weekday::Thu => "Quinta",
            Weekday::Fri => "Sexta",
            Weekday::Sat => "Sábado",
            Weekday::Sun => "Domingo",
        }
    }
}