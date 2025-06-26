// Dashboard functionality
document.addEventListener('DOMContentLoaded', function() {
    // Load user info
    const currentUser = JSON.parse(localStorage.getItem('currentUser'));
    if (currentUser) {
        const nameElement = document.getElementById('userName') || document.getElementById('teacherName');
        if (nameElement) {
            nameElement.textContent = currentUser.first_name;
        }
    }

    // Navigation handling
    const navItems = document.querySelectorAll('.nav-item');
    const sections = document.querySelectorAll('.content-section');

    navItems.forEach(item => {
        item.addEventListener('click', (e) => {
            e.preventDefault();
            
            // Remove active class from all items and sections
            navItems.forEach(nav => nav.classList.remove('active'));
            sections.forEach(section => section.classList.remove('active'));
            
            // Add active class to clicked item
            item.classList.add('active');
            
            // Show corresponding section
            const targetId = item.getAttribute('href').substring(1);
            const targetSection = document.getElementById(targetId);
            if (targetSection) {
                targetSection.classList.add('active');
            }
        });
    });
});

// Logout function
function logout() {
    localStorage.removeItem('authToken');
    localStorage.removeItem('currentUser');
    window.location.href = '/';
}

// Action functions
function joinLiveClass() {
    window.open('/api/classroom/live-session', '_blank');
}

function startClass() {
    window.open('/api/classroom/new-session', '_blank');
}

function createClass() {
    // Implementation for creating new class
    alert('Create new class functionality coming soon!');
}

function openBooks() {
    // Implementation for digital books
    alert('Digital books section coming soon!');
}

function watchVideos() {
    // Implementation for video watching
    alert('Video library coming soon!');
}

function uploadMaterial() {
    // Implementation for uploading materials
    alert('Material upload functionality coming soon!');
}
