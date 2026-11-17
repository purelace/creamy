mod diagnostic;

use creamy_xmlc::SemanticModel;
use dashmap::DashMap;
use tower_lsp::{Client, LanguageServer, LspService, Server, jsonrpc::Result, lsp_types::*};

#[derive(Debug)]
struct Backend {
    models: DashMap<Url, SemanticModel>,
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        match SemanticModel::compile(&params.text_document.text) {
            Ok(model) => {
                self.models.insert(params.text_document.uri, model);
            }
            Err(diag) => {
                panic!()
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {}

    async fn hover(&self, a: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                self.models
                    .get(&a.text_document_position_params.text_document.uri)
                    .unwrap()
                    .name()
                    .to_string(),
            )),
            range: None,
        }))
    }

    //async fn semantic_tokens_full(
    //    &self,
    //    params: SemanticTokensParams,
    //) -> Result<Option<SemanticTokensResult>> {
    //}
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        models: DashMap::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
