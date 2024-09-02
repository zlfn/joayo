use aws_config::Region;
use aws_sdk_s3::config::Credentials;

pub struct S3ClientCreator {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: String,
}

impl S3ClientCreator {
    pub fn create(&self) -> aws_sdk_s3::Client {
        let aws_credentials = Credentials::new(
            self.access_key_id.clone(),
            self.secret_access_key.clone(),
            None, None,
            "credentials"
        );

        let aws_config = aws_sdk_s3::Config::builder()
            .credentials_provider(aws_credentials)
            .endpoint_url(self.endpoint.clone())
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::v2024_03_28())
            .region(Region::from_static("auto"))
            .build();

        aws_sdk_s3::Client::from_conf(aws_config)
    }
}

