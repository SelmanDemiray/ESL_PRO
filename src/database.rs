use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use crate::models::{User, Classroom, DigitalBook};
use sqlx::Row;

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

        // 3. Create users table if not exists
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                email VARCHAR(255) NOT NULL UNIQUE,
                password_hash VARCHAR(255) NOT NULL,
                user_type user_type NOT NULL,
                first_name VARCHAR(100) NOT NULL,
                last_name VARCHAR(100) NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW(),
                is_active BOOLEAN DEFAULT TRUE,
                zoom_access_token VARCHAR(512),
                zoom_refresh_token VARCHAR(512),
                zoom_token_expiry TIMESTAMPTZ
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        // 4. Create classrooms table if not exists
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS classrooms (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR(255) NOT NULL,
                description TEXT,
                teacher_id UUID NOT NULL REFERENCES users(id),
                is_active BOOLEAN DEFAULT TRUE,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                zoom_meeting_id VARCHAR(64),
                zoom_join_url VARCHAR(512)
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        // 5. Add zoom fields to users table if not exist
        sqlx::query(
            r#"
            ALTER TABLE users
            ADD COLUMN IF NOT EXISTS zoom_access_token VARCHAR(512),
            ADD COLUMN IF NOT EXISTS zoom_refresh_token VARCHAR(512),
            ADD COLUMN IF NOT EXISTS zoom_token_expiry TIMESTAMPTZ;
            "#
        )
        .execute(&self.pool)
        .await?;

        // 6. Add zoom fields to classrooms table if not exist
        sqlx::query(
            r#"
            ALTER TABLE classrooms
            ADD COLUMN IF NOT EXISTS zoom_meeting_id VARCHAR(64),
            ADD COLUMN IF NOT EXISTS zoom_join_url VARCHAR(512);
            "#
        )
        .execute(&self.pool)
        .await?;

        // 7. Create meeting_requests table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS meeting_requests (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                classroom_id UUID NOT NULL REFERENCES classrooms(id),
                student_id UUID NOT NULL REFERENCES users(id),
                status VARCHAR(32) NOT NULL DEFAULT 'pending',
                created_at TIMESTAMPTZ DEFAULT NOW()
            );
            "#
        )
        .execute(&self.pool)
        .await?;

        // 8. Create digital_books table
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

        // 9. Create indexes
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

        // 10. Create lessons table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lessons (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                classroom_id UUID NOT NULL REFERENCES classrooms(id),
                teacher_id UUID NOT NULL REFERENCES users(id),
                title VARCHAR(255) NOT NULL,
                description TEXT,
                scheduled_at TIMESTAMPTZ NOT NULL,
                is_active BOOLEAN DEFAULT TRUE,
                chat_closed BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMPTZ DEFAULT NOW()
            );
            "#
        ).execute(&self.pool).await?;

        // 11. Create lesson_participants table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lesson_participants (
                lesson_id UUID NOT NULL REFERENCES lessons(id),
                user_id UUID NOT NULL REFERENCES users(id),
                is_muted BOOLEAN DEFAULT FALSE,
                PRIMARY KEY (lesson_id, user_id)
            );
            "#
        ).execute(&self.pool).await?;

        // 12. Create lesson_chat_messages table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lesson_chat_messages (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                lesson_id UUID NOT NULL REFERENCES lessons(id),
                user_id UUID NOT NULL REFERENCES users(id),
                username VARCHAR(100) NOT NULL,
                message TEXT NOT NULL,
                timestamp TIMESTAMPTZ DEFAULT NOW(),
                deleted BOOLEAN DEFAULT FALSE
            );
            "#
        ).execute(&self.pool).await?;

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
        .bind(&user.user_type) // bind as enum, not string
        .bind(&user.first_name)
        .bind(&user.last_name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, email, password_hash, user_type, 
               first_name, last_name, created_at, updated_at, is_active,
               zoom_access_token, zoom_refresh_token, zoom_token_expiry
               FROM users WHERE email = $1"#
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
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

    // --- Zoom meeting management ---

    pub async fn set_classroom_zoom_meeting(
        &self,
        classroom_id: Uuid,
        meeting_id: &str,
        join_url: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE classrooms SET zoom_meeting_id = $1, zoom_join_url = $2 WHERE id = $3"
        )
        .bind(meeting_id)
        .bind(join_url)
        .bind(classroom_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn clear_classroom_zoom_meeting(&self, classroom_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE classrooms SET zoom_meeting_id = NULL, zoom_join_url = NULL WHERE id = $1"
        )
        .bind(classroom_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // --- Meeting request management ---

    pub async fn create_meeting_request(
        &self,
        classroom_id: Uuid,
        student_id: Uuid,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO meeting_requests (id, classroom_id, student_id) VALUES (uuid_generate_v4(), $1, $2)"
        )
        .bind(classroom_id)
        .bind(student_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_pending_meeting_requests(
        &self,
        classroom_id: Uuid,
    ) -> anyhow::Result<Vec<crate::models::MeetingRequest>> {
        let requests = sqlx::query_as::<_, crate::models::MeetingRequest>(
            "SELECT * FROM meeting_requests WHERE classroom_id = $1 AND status = 'pending'"
        )
        .bind(classroom_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(requests)
    }

    pub async fn update_meeting_request_status(
        &self,
        request_id: Uuid,
        status: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE meeting_requests SET status = $1 WHERE id = $2"
        )
        .bind(status)
        .bind(request_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Lesson CRUD
    pub async fn create_lesson(&self, lesson: &crate::models::Lesson) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO lessons (id, classroom_id, teacher_id, title, description, scheduled_at, is_active, chat_closed, created_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)"
        )
        .bind(lesson.id)
        .bind(lesson.classroom_id)
        .bind(lesson.teacher_id)
        .bind(&lesson.title)
        .bind(&lesson.description)
        .bind(lesson.scheduled_at)
        .bind(lesson.is_active)
        .bind(lesson.chat_closed)
        .bind(lesson.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_lesson(&self, lesson_id: Uuid) -> anyhow::Result<Option<crate::models::Lesson>> {
        let lesson = sqlx::query_as::<_, crate::models::Lesson>(
            "SELECT * FROM lessons WHERE id = $1"
        )
        .bind(lesson_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(lesson)
    }

    pub async fn create_classroom(&self, classroom: &crate::models::Classroom) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO classrooms (id, name, description, teacher_id, is_active, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(classroom.id)
        .bind(&classroom.name)
        .bind(&classroom.description)
        .bind(classroom.teacher_id)
        .bind(classroom.is_active)
        .bind(classroom.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn create_material(&self, material: &crate::models::DigitalBook) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO digital_books (id, title, author, description, pdf_url, level, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(material.id)
        .bind(&material.title)
        .bind(&material.author)
        .bind(&material.description)
        .bind(&material.pdf_url)
        .bind(&material.level)
        .bind(material.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Lesson participants
    pub async fn add_lesson_participant(&self, lesson_id: Uuid, user_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO lesson_participants (lesson_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(lesson_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_participant_muted(&self, lesson_id: Uuid, user_id: Uuid, muted: bool) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE lesson_participants SET is_muted = $1 WHERE lesson_id = $2 AND user_id = $3"
        )
        .bind(muted)
        .bind(lesson_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn is_participant_muted(&self, lesson_id: Uuid, user_id: Uuid) -> anyhow::Result<bool> {
        let row = sqlx::query(
            "SELECT is_muted FROM lesson_participants WHERE lesson_id = $1 AND user_id = $2"
        )
        .bind(lesson_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.get::<bool, _>("is_muted")).unwrap_or(false))
    }

    // Lesson chat
    pub async fn add_chat_message(&self, msg: &crate::models::LessonChatMessage) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO lesson_chat_messages (id, lesson_id, user_id, username, message, timestamp, deleted)
             VALUES ($1,$2,$3,$4,$5,$6,$7)"
        )
        .bind(msg.id)
        .bind(msg.lesson_id)
        .bind(msg.user_id)
        .bind(&msg.username)
        .bind(&msg.message)
        .bind(msg.timestamp)
        .bind(msg.deleted)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_chat_message(&self, message_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE lesson_chat_messages SET deleted = TRUE WHERE id = $1"
        )
        .bind(message_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn close_lesson_chat(&self, lesson_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE lessons SET chat_closed = TRUE WHERE id = $1"
        )
        .bind(lesson_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Get all lessons for a teacher
    pub async fn get_lessons_by_teacher(&self, teacher_id: Uuid) -> anyhow::Result<Vec<crate::models::Lesson>> {
        let lessons = sqlx::query_as::<_, crate::models::Lesson>(
            "SELECT * FROM lessons WHERE teacher_id = $1 ORDER BY scheduled_at DESC"
        )
        .bind(teacher_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(lessons)
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}