pub struct UriSection<T> {
    execute_uri_section: fn(&Request<T>, &TcpStream, &Arc<Pool<Postgres>>) -> Future<Result<Option<UriSection<T>>, NinoverseApiError>>,
}

trait UriSectionFn<T> {
    async fn next(
        &self,
        request: Request<T>,
        stream: TcpStream,
        request_uri_parts: Vec<String>,
        pool: &Arc<Pool<Postgres>>,
    ) -> Result<(), NinoverseApiError>;
}

impl<T> UriSectionFn<T> for UriSection<T> {
    async fn next(
        &self,
        request: Request<T>,
        stream: TcpStream,
        request_uri_parts: Vec<String>,
        pool: &Arc<Pool<Postgres>>,
    ) -> Result<(), NinoverseApiError>;
        let next_section = (self.execute_uri_section)(request, stream, poll).await?;
        if let Some(next_section) = next_section {
            if !request_uri_parts.is_empty() {
                request_uri_parts.remove(0);
                (next_section.next(request, stream, request_uri_parts, pool)).await?;
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}


enum UriSectionType {
    Root,
    Branch,
    Leaf,
}
