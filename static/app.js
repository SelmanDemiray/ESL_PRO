// Authentication and UI management
let authToken = localStorage.getItem('authToken');
let currentUser = JSON.parse(localStorage.getItem('currentUser') || 'null');

// Modal management
function showLogin() {
    document.getElementById('loginModal').style.display = 'block';
    document.getElementById('registerModal').style.display = 'none';
}

function showRegister(userType = '') {
    document.getElementById('registerModal').style.display = 'block';
    document.getElementById('loginModal').style.display = 'none';
    if (userType) {
        document.getElementById('userType').value = userType;
    }
}

function closeModal(modalId) {
    document.getElementById(modalId).style.display = 'none';
}

// Close modals when clicking outside
window.onclick = function(event) {
    const loginModal = document.getElementById('loginModal');
    const registerModal = document.getElementById('registerModal');
    if (event.target === loginModal) {
        loginModal.style.display = 'none';
    }
    if (event.target === registerModal) {
        registerModal.style.display = 'none';
    }
}

// Login form handler
document.getElementById('loginForm').addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const email = document.getElementById('loginEmail').value;
    const password = document.getElementById('loginPassword').value;
    
    try {
        const response = await fetch('/api/auth/login', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ email, password }),
        });
        
        if (response.ok) {
            const data = await response.json();
            localStorage.setItem('authToken', data.token);
            localStorage.setItem('currentUser', JSON.stringify(data.user));
            window.location.href = '/api/dashboard';
        } else {
            alert('Invalid credentials. Please try again.');
        }
    } catch (error) {
        alert('Login failed. Please try again.');
    }
});

// Register form handler
document.getElementById('registerForm').addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const formData = {
        email: document.getElementById('registerEmail').value,
        password: document.getElementById('registerPassword').value,
        user_type: document.getElementById('userType').value,
        first_name: document.getElementById('firstName').value,
        last_name: document.getElementById('lastName').value,
    };
    
    if (formData.password.length < 8) {
        alert('Password must be at least 8 characters long.');
        return;
    }
    
    try {
        const response = await fetch('/api/auth/register', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(formData),
        });
        
        if (response.ok) {
            const data = await response.json();
            localStorage.setItem('authToken', data.token);
            localStorage.setItem('currentUser', JSON.stringify(data.user));
            window.location.href = '/api/dashboard';
        } else {
            alert('Registration failed. Email might already be in use.');
        }
    } catch (error) {
        alert('Registration failed. Please try again.');
    }
});

// Utility function for authenticated requests
async function authenticatedFetch(url, options = {}) {
    const token = localStorage.getItem('authToken');
    return fetch(url, {
        ...options,
        headers: {
            ...options.headers,
            'Authorization': `Bearer ${token}`,
        },
    });
}
