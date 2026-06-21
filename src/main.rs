mod app;
mod modules;
mod search;
mod ui;
mod widget;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

fn main() -> anyhow::Result<()> {
    let log_file = std::fs::File::create("debug.log")?;
    tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(log_file))
        .with_max_level(tracing::Level::DEBUG)
        .init();

    //直接运行stty进程对终端进行设置，-ixon表示关闭xon流控，即ctrl+s组合键暂停终端输出的xon协议，.status表示执行并等待命令返回值，线程阻塞
    //更常见是直接使用crossterm::terminal::enable_raw_mode()，在这里ratatui::init()已经启动了raw_mode
    let _ = std::process::Command::new("stty").args(["-ixon"]).status();
    let terminal = ratatui::init();
    let mut app = app::App::new();
    let result = run(&mut app, terminal);
    ratatui::restore();
    result
}
//接受一个状态控制器，和ratatui初始化后的终端，并将状态控制器中的内容，渲染到终端上
fn run(app: &mut app::App, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
    while app.running {
        //负责将当前状态渲染到终端中
        terminal.draw(|frame| ui::render(app, frame))?;
        //同步阻塞式的，功能为响应键盘按下事件，并传递给下面的状态控制器的对应处理函数
        let key = match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => key,
            _ => continue,
        };
        app.on_key_event(key);
    }
    Ok(())
}
