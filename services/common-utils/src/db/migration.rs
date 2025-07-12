#[macro_export]
macro_rules! run_db_migrations {
    ($pool:expr, $migrations_folder:literal) => {{
        use std::ops::DerefMut;
        refinery::embed_migrations!($migrations_folder);
        let mut conn = $pool.get().await?;
        let client = conn.deref_mut().deref_mut();
        let migration_report = migrations::runner().run_async(client).await?;
        println!("{:?}", migration_report);

        for migration in migration_report.applied_migrations() {
            println!(
                "Migration Applied: V{}_{}",
                migration.version(),
                migration.name(),
            );
        }
    }};
}
