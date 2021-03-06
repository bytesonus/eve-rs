use crate::{Context, Error, Middleware, NextHandler};
use std::fmt::Debug;
use tokio::fs;

#[derive(Clone)]
pub struct StaticFileServer {
	folder_path: String,
}

impl StaticFileServer {
	pub fn create(folder_path: &str) -> StaticFileServer {
		let folder_path = if let Some(stripped) = folder_path.strip_suffix('/') {
			stripped
		} else {
			folder_path
		}
		.to_string();
		StaticFileServer { folder_path }
	}

	pub async fn serve<TContext>(
		&self,
		mut context: TContext,
		next: NextHandler<TContext>,
	) -> Result<TContext, Error<TContext>>
	where
		TContext: Context + Debug + Send + Sync,
	{
		let file_location = format!("{}{}", self.folder_path, context.get_path());
		if is_file(&file_location).await {
			let content = fs::read(file_location).await?;
			context.body_bytes(&content);
			Ok(context)
		} else {
			next(context).await
		}
	}
}

#[async_trait::async_trait]
impl<TContext> Middleware<TContext> for StaticFileServer
where
	TContext: 'static + Context + Debug + Send + Sync,
{
	async fn run_middleware(
		&self,
		context: TContext,
		next: NextHandler<TContext>,
	) -> Result<TContext, Error<TContext>> {
		self.serve(context, next).await
	}
}

pub fn static_server(folder_path: &str) -> StaticFileServer {
	StaticFileServer::create(folder_path)
}

async fn is_file(path: &str) -> bool {
	let metadata = fs::metadata(path).await;
	if metadata.is_err() {
		return false;
	}
	metadata.unwrap().is_file()
}
