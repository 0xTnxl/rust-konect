class ChatApp {
    constructor() {
        this.token = localStorage.getItem('auth_token');
        this.currentUser = null;
        this.currentRoom = null;
        this.websocket = null;
        this.rooms = [];
        
        this.initializeElements();
        this.attachEventListeners();
        
        if (this.token) {
            this.showChatInterface();
            this.loadRooms();
        } else {
            this.showAuthModal();
        }
    }
    
    initializeElements() {
        // Auth elements
        this.authModal = document.getElementById('auth-modal');
        this.authForm = document.getElementById('auth-form');
        this.authTitle = document.getElementById('auth-title');
        this.authSubmit = document.getElementById('auth-submit');
        this.authToggle = document.getElementById('auth-toggle');
        this.toggleAuth = document.getElementById('toggle-auth');
        this.usernameField = document.getElementById('username-field');
        
        // Chat elements
        this.chatContainer = document.getElementById('chat-container');
        this.currentUserSpan = document.getElementById('current-user');
        this.logoutBtn = document.getElementById('logout-btn');
        this.roomsList = document.getElementById('rooms-list');
        this.createRoomBtn = document.getElementById('create-room-btn');
        this.currentRoomName = document.getElementById('current-room-name');
        this.messagesList = document.getElementById('messages-list');
        this.messageInput = document.getElementById('message-input');
        this.sendBtn = document.getElementById('send-btn');
        this.fileBtn = document.getElementById('file-btn');
        this.fileInput = document.getElementById('file-input');
        
        // Room creation modal
        this.createRoomModal = document.getElementById('create-room-modal');
        this.createRoomForm = document.getElementById('create-room-form');
        this.cancelRoomBtn = document.getElementById('cancel-room-btn');
    }
    
    attachEventListeners() {
        // Auth form
        this.authForm.addEventListener('submit', (e) => this.handleAuth(e));
        this.toggleAuth.addEventListener('click', (e) => this.toggleAuthMode(e));
        
        // Chat interface
        this.logoutBtn.addEventListener('click', () => this.logout());
        this.createRoomBtn.addEventListener('click', () => this.showCreateRoomModal());
        this.messageInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') this.sendMessage();
        });
        this.sendBtn.addEventListener('click', () => this.sendMessage());
        this.fileBtn.addEventListener('click', () => this.fileInput.click());
        this.fileInput.addEventListener('change', (e) => this.handleFileUpload(e));
        
        // Room creation
        this.createRoomForm.addEventListener('submit', (e) => this.createRoom(e));
        this.cancelRoomBtn.addEventListener('click', () => this.hideCreateRoomModal());
    }
    
    async handleAuth(e) {
        e.preventDefault();
        const formData = new FormData(this.authForm);
        const isLogin = this.authTitle.textContent === 'Login';
        
        const data = {
            email: formData.get('email'),
            password: formData.get('password')
        };
        
        if (!isLogin) {
            data.username = formData.get('username');
        }
        
        try {
            const response = await fetch(`/api/auth/${isLogin ? 'login' : 'register'}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(data)
            });
            
            if (response.ok) {
                const result = await response.json();
                this.token = result.token;
                this.currentUser = result.user;
                localStorage.setItem('auth_token', this.token);
                
                this.showChatInterface();
                this.loadRooms();
            } else {
                const error = await response.json();
                this.showError(error.error || 'Authentication failed');
            }
        } catch (error) {
            this.showError('Network error: ' + error.message);
        }
    }
    
    toggleAuthMode(e) {
        e.preventDefault();
        const isLogin = this.authTitle.textContent === 'Login';
        
        if (isLogin) {
            this.authTitle.textContent = 'Register';
            this.authSubmit.textContent = 'Register';
            this.authToggle.innerHTML = 'Already have an account? <a href="#" id="toggle-auth">Login</a>';
            this.usernameField.style.display = 'block';
            this.usernameField.querySelector('input').required = true;
        } else {
            this.authTitle.textContent = 'Login';
            this.authSubmit.textContent = 'Login';
            this.authToggle.innerHTML = 'Don\'t have an account? <a href="#" id="toggle-auth">Register</a>';
            this.usernameField.style.display = 'none';
            this.usernameField.querySelector('input').required = false;
        }
        
        // Re-attach event listener
        this.toggleAuth = document.getElementById('toggle-auth');
        this.toggleAuth.addEventListener('click', (e) => this.toggleAuthMode(e));
    }
    
    showAuthModal() {
        this.authModal.style.display = 'flex';
        this.chatContainer.style.display = 'none';
    }
    
    showChatInterface() {
        this.authModal.style.display = 'none';
        this.chatContainer.style.display = 'block';
        
        if (this.currentUser) {
            this.currentUserSpan.textContent = this.currentUser.username;
        }
    }
    
    logout() {
        this.token = null;
        this.currentUser = null;
        this.currentRoom = null;
        localStorage.removeItem('auth_token');
        
        if (this.websocket) {
            this.websocket.close();
            this.websocket = null;
        }
        
        this.showAuthModal();
    }
    
    async loadRooms() {
        try {
            const response = await fetch('/api/rooms', {
                headers: { 'Authorization': `Bearer ${this.token}` }
            });
            
            if (response.ok) {
                this.rooms = await response.json();
                this.renderRooms();
            }
        } catch (error) {
            this.showError('Failed to load rooms: ' + error.message);
        }
    }
    
    renderRooms() {
        this.roomsList.innerHTML = '';
        
        this.rooms.forEach(room => {
            const li = document.createElement('li');
            li.className = 'room-item';
            li.dataset.roomId = room.id;
            
            li.innerHTML = `
                <div class="room-name">${room.name}</div>
                ${room.description ? `<div class="room-description">${room.description}</div>` : ''}
            `;
            
            li.addEventListener('click', () => this.selectRoom(room));
            this.roomsList.appendChild(li);
        });
    }
    
    selectRoom(room) {
        // Update UI
        document.querySelectorAll('.room-item').forEach(item => {
            item.classList.remove('active');
        });
        document.querySelector(`[data-room-id="${room.id}"]`).classList.add('active');
        
        this.currentRoom = room;
        this.currentRoomName.textContent = room.name;
        
        // Enable message input
        this.messageInput.disabled = false;
        this.sendBtn.disabled = false;
        this.fileBtn.disabled = false;
        
        // Connect WebSocket
        this.connectWebSocket(room.id);
        
        // Load message history
        this.loadMessages(room.id);
    }
    
    connectWebSocket(roomId) {
        if (this.websocket) {
            this.websocket.close();
        }
        
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws/${roomId}`;
        
        this.websocket = new WebSocket(wsUrl);
        
        this.websocket.onopen = () => {
            console.log('WebSocket connected');
            this.sendWebSocketMessage('join_room', { room_id: roomId });
        };
        
        this.websocket.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                this.displayMessage(message);
            } catch (error) {
                console.error('Failed to parse WebSocket message:', error);
            }
        };
        
        this.websocket.onclose = () => {
            console.log('WebSocket disconnected');
        };
        
        this.websocket.onerror = (error) => {
            console.error('WebSocket error:', error);
        };
    }
    
    sendWebSocketMessage(type, data) {
        if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
            this.websocket.send(JSON.stringify({
                message_type: type,
                data: data
            }));
        }
    }
    
    async loadMessages(roomId) {
        try {
            const response = await fetch(`/api/rooms/${roomId}/messages?limit=50`, {
                headers: { 'Authorization': `Bearer ${this.token}` }
            });
            
            if (response.ok) {
                const messages = await response.json();
                this.messagesList.innerHTML = '';
                messages.reverse().forEach(message => this.displayMessage(message));
                this.scrollToBottom();
            }
        } catch (error) {
            this.showError('Failed to load messages: ' + error.message);
        }
    }
    
    displayMessage(message) {
        const messageEl = document.createElement('div');
        messageEl.className = 'message';
        
        const isOwnMessage = this.currentUser && message.user_id === this.currentUser.id;
        messageEl.classList.add(isOwnMessage ? 'own' : 'other');
        
        const timestamp = new Date(message.created_at).toLocaleTimeString();
        
        if (message.message_type === 'file') {
            const fileData = JSON.parse(message.content);
            messageEl.innerHTML = `
                <div class="message-header">${isOwnMessage ? 'You' : 'User'}</div>
                <div class="message-content">
                    <div class="file-item">
                        <a href="${fileData.url}" class="file-link" target="_blank">
                            📎 ${fileData.filename} (${this.formatFileSize(fileData.size)})
                        </a>
                    </div>
                </div>
                <div class="message-time">${timestamp}</div>
            `;
        } else {
            messageEl.innerHTML = `
                <div class="message-header">${isOwnMessage ? 'You' : 'User'}</div>
                <div class="message-content">${this.escapeHtml(message.content)}</div>
                <div class="message-time">${timestamp}</div>
            `;
        }
        
        this.messagesList.appendChild(messageEl);
        this.scrollToBottom();
    }
    
    async sendMessage() {
        const content = this.messageInput.value.trim();
        if (!content || !this.currentRoom) return;
        
        try {
            const response = await fetch(`/api/rooms/${this.currentRoom.id}/messages`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.token}`
                },
                body: JSON.stringify({
                    content: content,
                    message_type: 'text'
                })
            });
            
            if (response.ok) {
                this.messageInput.value = '';
                // Message will be displayed via WebSocket
            } else {
                const error = await response.json();
                this.showError(error.error || 'Failed to send message');
            }
        } catch (error) {
            this.showError('Network error: ' + error.message);
        }
    }
    
    async handleFileUpload(e) {
        const files = Array.from(e.target.files);
        if (files.length === 0) return;
        
        for (const file of files) {
            const formData = new FormData();
            formData.append('file', file);
            
            try {
                const response = await fetch('/api/upload', {
                    method: 'POST',
                    headers: { 'Authorization': `Bearer ${this.token}` },
                    body: formData
                });
                
                if (response.ok) {
                    const uploadResult = await response.json();
                    
                    // Send file message
                    await fetch(`/api/rooms/${this.currentRoom.id}/messages`, {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                            'Authorization': `Bearer ${this.token}`
                        },
                        body: JSON.stringify({
                            content: JSON.stringify(uploadResult),
                            message_type: 'file'
                        })
                    });
                } else {
                    const error = await response.json();
                    this.showError(error.error || 'Failed to upload file');
                }
            } catch (error) {
                this.showError('Upload error: ' + error.message);
            }
        }
        
        // Clear file input
        this.fileInput.value = '';
    }
    
    showCreateRoomModal() {
        this.createRoomModal.style.display = 'flex';
    }
    
    hideCreateRoomModal() {
        this.createRoomModal.style.display = 'none';
        this.createRoomForm.reset();
    }
    
    async createRoom(e) {
        e.preventDefault();
        const formData = new FormData(this.createRoomForm);
        
        const data = {
            name: formData.get('name'),
            description: formData.get('description') || null
        };
        
        try {
            const response = await fetch('/api/rooms', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.token}`
                },
                body: JSON.stringify(data)
            });
            
            if (response.ok) {
                const newRoom = await response.json();
                this.rooms.unshift(newRoom);
                this.renderRooms();
                this.hideCreateRoomModal();
                this.selectRoom(newRoom);
            } else {
                const error = await response.json();
                this.showError(error.error || 'Failed to create room');
            }
        } catch (error) {
            this.showError('Network error: ' + error.message);
        }
    }
    
    scrollToBottom() {
        const container = document.getElementById('messages-container');
        container.scrollTop = container.scrollHeight;
    }
    
    showError(message) {
        // Create or update error message element
        let errorEl = document.querySelector('.error-message');
        if (!errorEl) {
            errorEl = document.createElement('div');
            errorEl.className = 'error-message';
            document.body.insertBefore(errorEl, document.body.firstChild);
        }
        
        errorEl.textContent = message;
        
        // Auto-hide after 5 seconds
        setTimeout(() => {
            if (errorEl.parentNode) {
                errorEl.parentNode.removeChild(errorEl);
            }
        }, 5000);
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    formatFileSize(bytes) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
}

// Initialize the app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new ChatApp();
});