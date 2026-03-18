# Agenda Sync 📅

Uma interface de terminal (TUI) moderna e performática, desenvolvida em Rust, para gestão de horários acadêmicos e monitorias. O projeto foi desenhado para ser leve, funcional e visualmente intuitivo.

> [!IMPORTANT]
> **Aviso de Autoria:** Este projeto foi idealizado, estruturado e validado por **Benjamin (@the0hax)**. O desenvolvimento do código e a documentação contaram com o auxílio de Inteligência Artificial (Gemini/Google) para otimização de sintaxe e implementação de padrões de engenharia em Rust. A lógica de negócio, regras de privacidade e arquitetura funcional são de autoria do usuário.

## ✨ Funcionalidades

- **Visão Multi-Aba**: Alternância rápida entre grade de Aulas, Monitoria e Interseção Geral.
- **Detecção de Conflitos**: Sistema inteligente que destaca colisões de horários em cores vibrantes.
- **Persistência de Dados**: Carregamento e salvamento automático via JSON (`agenda_sync.json`).
- **Interface Responsiva**: Layout adaptável feito com Ratatui.
- **Splash Screen**: Tela de abertura em arte ASCII para uma experiência profissional.

## 📦 Bibliotecas Utilizadas

- **[Ratatui](https://ratatui.rs/)**: Engine principal para renderização da interface TUI.
- **[Crossterm](https://github.com/crossterm-rs/crossterm)**: Manipulação de eventos de teclado e controle de terminal multiplataforma.
- **[Serde](https://serde.rs/) & [Serde_JSON](https://github.com/serde-rs/json)**: Serialização e persistência de dados no disco rígido.
- **[Chrono](https://github.com/chronotope/chrono)**: Gestão precisa de data, hora e dias da semana.

## 🚀 Instalação

Para utilizar a agenda como um comando nativo do seu sistema:

```bash
cargo install --path .
```

## 🛠️ Como Usar

Após a instalação, você pode chamar a aplicação de qualquer diretório:

```bash
# Iniciar a agenda
agenda

# Iniciar limpando os dados salvos (reset de fábrica)
agenda --reset
```

### Atalhos de Teclado

| Tecla | Ação |
| :--- | :--- |
| **Setas** | Navegação entre as células da grade |
| **1, 2, 3** | Alternar entre Abas (Aulas / Monitoria / Interseção) |
| **Enter** | Editar o conteúdo da célula selecionada |
| **A** | Adicionar uma nova linha de horário |
| **M** | Modificar o texto da linha de horário atual |
| **Delete** | Limpar os dados da célula atual |
| **Q** | Salvar alterações e sair da aplicação |

---
Desenvolvido por Benjamin (@the0hax) - Estudante de Ciência da Computação.