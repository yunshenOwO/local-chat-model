use kovi::tokio::sync::Mutex;
use kovi::{AllMsgEvent, PluginBuilder, RuntimeBot};
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::completion::GenerationContext;
use ollama_rs::Ollama;
use std::sync::{Arc, LazyLock};
use kovi::log::info;

static MEMORY: LazyLock<Mutex<Option<GenerationContext>>> = LazyLock::new(|| {Mutex::new(None)});


#[kovi::plugin]
async fn main() {
    let bot = PluginBuilder::get_runtime_bot();
    PluginBuilder::on_group_msg(move |event| group_msg_dispose(event, bot.clone()));
}

async fn group_msg_dispose(event: Arc<AllMsgEvent>, bot: Arc<RuntimeBot>) {
    let text = match event.borrow_text() {
        Some(v) => {
            v
        },
        None => return,
    };
    llama_bot(text, event.clone(), bot).await;

}

async fn llama_bot(text: &str, event:Arc<AllMsgEvent>, bot:Arc<RuntimeBot>){
    if text.starts_with("芸汐") || event.message.contains("at"){
        let message_content = text.replace("芸汐","");
        let llama = Ollama::default();
        if message_content.is_empty() {
            return;
        }
        let session = format!("{}说: {}", event.get_sender_nickname(), message_content);
        let mut request = GenerationRequest::new("llama3.2".into(), session.to_string())
            .system("你将扮演一个名叫芸汐的女孩子，说话尽量简短，不要加杂乱的符号，如果想表达心情请使用emoji，要活泼可爱,如果不想说话了就回复[sp]".to_string());
        if let Some(context) = MEMORY.lock().await.take() {
            request = request.context(context);
        }
        let response = llama.generate(request).await.unwrap();
        if !response.response.contains("[sp]") {
            bot.send_group_msg(event.group_id.unwrap(), response.response);
            let mut context = MEMORY.lock().await;
            *context = response.context;
        }else {
            info!("{}", "当前ai不想理你");
        }
    }
}