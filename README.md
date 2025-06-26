# ESL Learning Platform

A comprehensive English as a Second Language (ESL) learning platform built with Rust, featuring secure authentication, live classrooms, and interactive learning materials.

## Features

- 🔐 **Secure Authentication**: JWT-based auth with bcrypt password hashing
- 👥 **Role-based Access**: Separate dashboards for students and teachers
- 🎓 **Live Classrooms**: Real-time WebSocket-powered classroom sessions
- 📚 **Digital Library**: Access to ESL books and materials
- 🎥 **Video Learning**: Synchronized video watching experiences
- 📊 **Progress Tracking**: Monitor learning progress and statistics
- 🛡️ **Enterprise Security**: Production-ready security measures

## Quick Start

1. **Clone and Setup**:
   ```bash
   cd /root/mylove
   chmod +x setup.sh
   ./setup.sh
   ```

2. **Access the Platform**:
   - The setup script will display your public URL
   - Register as either a student or teacher
   - Access your personalized dashboard

## Architecture

- **Backend**: Rust with Axum framework
- **Database**: PostgreSQL with proper indexing
- **Authentication**: JWT tokens with secure password hashing
- **Real-time**: WebSocket support for live sessions
- **Frontend**: Modern HTML/CSS/JavaScript
- **Deployment**: Docker Compose with random port assignment

## Security Features

- Password hashing with bcrypt
- JWT token-based authentication
- SQL injection prevention with SQLx
- CORS protection
- Input validation and sanitization
- Secure session management

## Development

To run in development mode:
```bash
# Install Rust dependencies
cargo build

# Start PostgreSQL
docker-compose up postgres -d

# Run the application
cargo run
```

## Production Deployment

The platform is designed for production use with:
- Proper error handling
- Logging with tracing
- Database connection pooling
- Docker containerization
- Environment-based configuration

## API Endpoints

- `POST /api/auth/register` - User registration
- `POST /api/auth/login` - User login
- `GET /api/dashboard` - User dashboard (authenticated)
- `GET /api/classroom/:id` - Classroom access
- `GET /ws` - WebSocket connection for real-time features

## Contributing

This is a production-ready ESL learning platform. Contributions are welcome for additional features like:
- Advanced progress analytics
- Mobile app integration
- AI-powered language assessment
- Multi-language support
