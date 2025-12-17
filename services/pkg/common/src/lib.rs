use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Trait for generating UUIDs.
pub trait UuidGenerator: Send + Sync + 'static {
    /// Generates a new UUID.
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

/// Default UUID v4 generator implementation.
pub struct UuidV4Generator;

impl UuidGenerator for UuidV4Generator {}

/// Trait for providing the current UTC time.
pub trait Now: Send + Sync + 'static {
    /// Returns the current UTC time.
    fn now() -> DateTime<Utc>;
}

/// Implementation that returns the actual current system time.
pub struct SystemNow;

impl Now for SystemNow {
    fn now() -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use super::*;

    /// Mock UUID generator for testing.
    #[derive(Default)]
    pub struct MockUuidGenerator {
        /// The UUID to return from generate().
        pub uuid: Uuid,
    }

    impl MockUuidGenerator {
        /// Creates a new mock generator with a default test UUID.
        pub fn new() -> Self {
            Self {
                uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            }
        }

        /// Creates a new mock generator with the specified UUID.
        pub fn with_uuid(uuid: Uuid) -> Self {
            Self { uuid }
        }
    }

    impl UuidGenerator for MockUuidGenerator {
        fn generate(&self) -> Uuid {
            self.uuid
        }
    }

    /// Mock time provider for testing.
    pub struct MockNow {
        pub time: DateTime<Utc>,
    }

    impl MockNow {
        /// Creates a new mock with the specified time.
        pub fn new(time: DateTime<Utc>) -> Self {
            Self { time }
        }

        /// Creates a new mock with a default test time (2020-01-01 00:00:00 UTC).
        pub fn default_time() -> Self {
            Self {
                time: DateTime::from_timestamp(1577836800, 0).unwrap(), // 2020-01-01 00:00:00 UTC
            }
        }
    }

    impl Now for MockNow {
        fn now() -> DateTime<Utc> {
            DateTime::from_timestamp(1577836800, 0).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_v4_generator() {
        let generator = UuidV4Generator;
        let uuid1 = generator.generate();
        let uuid2 = generator.generate();

        assert_ne!(uuid1, uuid2);
        assert_eq!(uuid1.get_version(), Some(uuid::Version::Random));
    }

    #[test]
    fn test_system_now() {
        let now1 = SystemNow::now();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let now2 = SystemNow::now();

        assert!(now2 > now1);
    }

    #[cfg(feature = "mock")]
    #[test]
    fn test_mock_uuid_generator() {
        use mock::MockUuidGenerator;

        let test_uuid = Uuid::parse_str("12345678-1234-5678-9abc-123456789abc").unwrap();
        let generator = MockUuidGenerator::with_uuid(test_uuid);

        assert_eq!(generator.generate(), test_uuid);
        assert_eq!(generator.generate(), test_uuid);
    }

    #[cfg(feature = "mock")]
    #[test]
    fn test_mock_uuid_generator_default() {
        use mock::MockUuidGenerator;

        let generator = MockUuidGenerator::new();
        let expected_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

        assert_eq!(generator.generate(), expected_uuid);
    }

    #[cfg(feature = "mock")]
    #[test]
    fn test_mock_now() {
        use mock::MockNow;

        let test_time = DateTime::from_timestamp(1234567890, 0).unwrap();
        let mock_now = MockNow::new(test_time);

        let _ = mock_now;
    }
}
