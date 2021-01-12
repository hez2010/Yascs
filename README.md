# Yet Another Simple Chat Server

## Features
- Messaging
- Friend system
- Realtime message streaming
- Super fast and low resource usage

## Prerequsities
- Rust 1.49+
- PostgreSQL 13+

## Build
```bash
cargo build --release
```

## Database Migrations
Add PostgreSQL `bin` directory to environment variable `PATH`, and `lib` directory to environment variable `LIB`, and then install `diesel-cli`:

```bash
cargo install diesel_cli --no-default-features --features postgres
```

After installation, run:

```bash
diesel migration run
```

## Run
Configure database connection url via environment variable first:

Bash：
```bash
export DATABASE_URL=postgres://username:password@address/database
```

CMD：
```bash
set DATABASE_URL=postgres://username:password@address/database
```

PowerShell：
```bash
$env:DATABASE_URL="postgres://username:password@address/database"
```

Add PostgreSQL `bin` directory to environment variable `PATH`, and `lib` directory to environment variable `LIB`, and then you're ready to go.

## API
### Users `/api/user`
#### Login `/login`
```
HTTP POST
JSON { username: string, password: string }
```
#### Register `/register`
```
HTTP POST
JSON { username: string, password: string, confirmPassword: string, email: string }
```
#### Logout `/logout`
```
HTTP POST
```
#### Reset Password `/password`
```
HTTP POST
JSON { originalPassword: string, newPassword: string, confirmPassword }
```
#### Update Profiles `/profiles`
```
HTTP POST
JSON { username: string, email: string, phone: string, location: string, age: number, gender: number, avatar: string }
```
#### Get Current Profiles `/profiles`
```
HTTP GET
```
#### Get User Profiles `/profiles/{userId}`
```
HTTP GET
```
#### Search Users `/search?patterns=string`
```
HTTP GET
```
#### Get Friends List `/friends`
```
HTTP GET
```
#### Add as Friend `/friends/{userId}`
```
HTTP POST
```
#### Delete Friend `/friends/{userId}`
```
HTTP DELETE
```

### Chat `/api/message`
#### List Chat Sessions `/list`
```
HTTP GET
```
#### Get Session History `/history/{userId}`
```
HTTP GET
```
#### Send Message `/send`
```
HTTP POST
JSON { toUser: number, messageType: number, message: string, quoteId: number }
```
#### Set Read Message `/read/{messageId}`
```
HTTP POST
```
#### Streaming Message `/stream`
```
WebSocket
```

### Response
```
JSON { status: boolean, code: number, data: object?, message: string? }
```
