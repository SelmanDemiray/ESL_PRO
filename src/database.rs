use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use crate::models::{User, Classroom, DigitalBook};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let mut retries = 10;
        loop {
            match PgPoolOptions::new()
                .max_connections(20)
                .connect(database_url)
                .await
            {
                Ok(pool) => break Ok(Database { pool }),
                Err(e) if retries > 0 => {
                    eprintln!("Database not ready yet: {e}. Retrying...");
                    retries -= 1;
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        // 1. Create extension
        sqlx::query(r#"CREATE EXTENSION IF NOT EXISTS "uuid-ossp";"#)
            .execute(&self.pool)
            .await?;

        // 2. Create user_type enum if not exists
        sqlx::query(
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_type') THEN
                    CREATE TYPE user_type AS ENUM ('student', 'teacher', 'admin');
                END IF;
            END$$;
            "#
        )
        .execute(&self.pool)
        .await?;

        // 3. Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                user_type user_type NOT NULL,
                first_name VARCHAR(100) NOT NULL,
                last_name VARCHAR(100) NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW(),
                is_active BOOLEAN DEFAULT TRUE
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        // 4. Create classrooms table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS classrooms (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR(255) NOT NULL,
                description TEXT,
                teacher_id UUID NOT NULL REFERENCES users(id),
                is_active BOOLEAN DEFAULT TRUE,
                created_at TIMESTAMPTZ DEFAULT NOW()
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        // 5. Create digital_books table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS digital_books (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                title VARCHAR(255) NOT NULL,
                author VARCHAR(255) NOT NULL,
                description TEXT,
                pdf_url VARCHAR(500) NOT NULL,
                level VARCHAR(50) NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW()
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        // 6. Create indexes
        sqlx::query(
            r#"CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);"#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"CREATE INDEX IF NOT EXISTS idx_users_type ON users(user_type);"#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_user(&self, user: &User) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, user_type, first_name, last_name)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.user_type.to_string().to_lowercase())
        .bind(&user.first_name)
        .bind(&user.last_name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, email, password_hash, user_type as "user_type: String", 
               first_name, last_name, created_at, updated_at, is_active 
               FROM users WHERE email = $1"#
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?
        .map(|mut u| {
            // Convert user_type string to enum
            u.user_type = u.user_type_from_str();
            u
        });

        Ok(user)
    }

    pub async fn get_classrooms_by_teacher(&self, teacher_id: Uuid) -> anyhow::Result<Vec<Classroom>> {
        let classrooms = sqlx::query_as::<_, Classroom>(
            "SELECT * FROM classrooms WHERE teacher_id = $1 AND is_active = TRUE"
        )
        .bind(teacher_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(classrooms)
    }

    pub async fn get_all_books(&self) -> anyhow::Result<Vec<DigitalBook>> {
        let books = sqlx::query_as::<_, DigitalBook>(
            "SELECT * FROM digital_books ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(books)
    }
}
