# Rust Konect - Real-time Chat Application

A modern, production-ready chat application built with Rust, featuring real-time messaging, user authentication, file sharing, and XMPP bridge capabilities.

## Features

- **Real-time Messaging**: WebSocket-based instant messaging with broadcast support
- **User Authentication**: Secure JWT-based authentication system
- **Chat Rooms**: Create and manage multiple chat rooms
- **Message History**: Persistent message storage with pagination
- **File Sharing**: Upload and share files with other users
- **XMPP Bridge**: Bridge support for connecting with XMPP networks (placeholder implementation)
- **Responsive UI**: Mobile-friendly web interface
- **Production Ready**: Docker containerization, error handling, and security features

## Technology Stack

### Backend
- **Rust** with Tokio async runtime
- **Axum** - Modern web framework
- **tokio-tungstenite** - WebSocket implementation
- **SQLx** - Async PostgreSQL driver with migrations
- **PostgreSQL** - Primary database
- **xmpp-rs** - XMPP protocol support (placeholder)
- **JWT** - Authentication tokens
- **bcrypt** - Password hashing

### Frontend
- **HTML5/CSS3/JavaScript** - Responsive web interface
- **WebSocket API** - Real-time communication
- **File Upload API** - Multi-file upload support

## Quick Start

### Using Docker Compose (Recommended)

1. Clone the repository:
```bash
git clone https://github.com/0xTnxl/rust-konect.git
cd rust-konect
```

2. Start the application:
```bash
docker-compose up --build
```

3. Open your browser and navigate to `http://localhost:3000`

### Manual Setup

#### Prerequisites
- Rust 1.70+ 
- PostgreSQL 13+
- Node.js (for development tools, optional)

#### Database Setup
1. Install PostgreSQL and create a database:
```sql
CREATE DATABASE rust_konect;
```

2. Copy environment configuration:
```bash
cp .env.example .env
```

3. Update database URL in `.env`:
```
DATABASE_URL=postgresql://postgres:password@localhost:5432/rust_konect
```

#### Running the Application
1. Build and run the backend:
```bash
cd backend
cargo run
```

2. Open your browser and navigate to `http://localhost:3000`

## Usage

### Getting Started
1. **Register**: Create a new account with username, email, and password
2. **Login**: Use your credentials to access the chat interface
3. **Create Room**: Click the "+" button to create a new chat room
4. **Join Conversations**: Select a room from the sidebar to start chatting
5. **Send Messages**: Type in the message input and press Enter or click Send
6. **Share Files**: Click the paperclip icon to upload and share files

### API Endpoints

#### Authentication
- `POST /api/auth/register` - Register a new user
- `POST /api/auth/login` - Login with existing credentials

#### Chat Rooms
- `GET /api/rooms` - List all available rooms
- `POST /api/rooms` - Create a new room
- `GET /api/rooms/:id/messages` - Get message history for a room
- `POST /api/rooms/:id/messages` - Send a message to a room

#### File Upload
- `POST /api/upload` - Upload files (multipart/form-data)

#### WebSocket
- `WS /ws/:room_id` - Real-time messaging connection

### WebSocket Message Format
```json
{
  "message_type": "chat_message",
  "data": {
    "room_id": "uuid",
    "content": "Hello, world!",
    "message_type": "text"
  }
}
```

## Development

### Project Structure
```
rust-konect/
├── backend/                 # Rust backend application
│   ├── src/
│   │   ├── main.rs         # Main application entry point
│   │   ├── auth.rs         # Authentication logic
│   │   ├── chat.rs         # Chat room management
│   │   ├── database.rs     # Database initialization
│   │   ├── error.rs        # Error handling
│   │   ├── models.rs       # Data models
│   │   ├── websocket.rs    # WebSocket handling
│   │   └── xmpp_bridge.rs  # XMPP bridge (placeholder)
│   ├── migrations/         # Database migrations
│   └── Cargo.toml         # Rust dependencies
├── frontend/               # Frontend web application
│   ├── index.html         # Main HTML page
│   └── static/
│       ├── style.css      # Styles
│       └── app.js         # JavaScript application
├── docker-compose.yml     # Docker development setup
├── Dockerfile            # Production container
└── README.md
```

### Running Tests
```bash
cd backend
cargo test
```

### Database Migrations
Migrations are automatically run on startup. To create new migrations:
```bash
cd backend
sqlx migrate add <migration_name>
```

### Environment Variables
- `DATABASE_URL` - PostgreSQL connection string
- `JWT_SECRET` - Secret key for JWT tokens
- `RUST_LOG` - Log level (debug, info, warn, error)
- `XMPP_SERVER` - XMPP server address (optional)
- `MAX_FILE_SIZE` - Maximum file upload size in bytes

## Security Features

- **Password Hashing**: bcrypt with salt
- **JWT Authentication**: Secure token-based authentication
- **Input Validation**: Request validation and sanitization
- **SQL Injection Protection**: Parameterized queries with SQLx
- **XSS Protection**: HTML escaping in frontend
- **CORS Configuration**: Configurable cross-origin requests
- **File Upload Validation**: File type and size restrictions

## Production Deployment

### Docker Deployment
The application includes a multi-stage Dockerfile for optimal production builds:

```bash
# Build and run with Docker Compose
docker-compose -f docker-compose.prod.yml up --build -d
```

### Environment Configuration
For production, ensure you:
1. Change the JWT secret to a secure random value
2. Use a production PostgreSQL instance
3. Configure proper CORS origins
4. Set up reverse proxy (nginx) for HTTPS
5. Configure file upload limits and storage

### Performance Considerations
- Database connection pooling is configured via SQLx
- WebSocket connections are efficiently managed
- Static files are served efficiently
- File uploads are streamed to prevent memory issues

## XMPP Bridge (Future Enhancement)

The application includes a placeholder XMPP bridge module for future integration with XMPP networks. This would allow:
- Bridging chat rooms with XMPP MUC (Multi-User Chat)
- Cross-protocol messaging
- Integration with existing XMPP infrastructure

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Ensure all tests pass: `cargo test`
5. Commit your changes: `git commit -am 'Add feature'`
6. Push to the branch: `git push origin feature-name`
7. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For issues, questions, or contributions, please:
- Open an issue on GitHub
- Check the documentation
- Review existing issues and discussions

---

Built with ❤️ in Rust