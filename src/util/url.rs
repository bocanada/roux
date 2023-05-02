use url::Url;

/// Builds a url for read only Reddit access.
pub fn build_url(dest: &str) -> String {
    format!("https://www.reddit.com/{}/.json", dest)
}

/// Builds a url for OAuth Reddit access.
pub fn build_oauth(dest: &str) -> String {
    format!("https://oauth.reddit.com/{}/.json", dest)
}

pub(crate) trait JoinSegmentsExt {
    fn join_segments(&self, segments: &[&str]) -> Self;
}

impl JoinSegmentsExt for Url {
    fn join_segments(&self, segments: &[&str]) -> Self {
        let mut url = self.clone();
        {
            let mut url_segments = url.path_segments_mut().unwrap();
            url_segments.pop_if_empty();

            for segment in segments {
                url_segments.push(segment);
            }
        }

        url
    }
}
