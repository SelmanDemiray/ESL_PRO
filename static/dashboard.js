// Dashboard functionality
document.addEventListener('DOMContentLoaded', function() {
    // Load user info
    const currentUser = JSON.parse(localStorage.getItem('currentUser'));
    if (currentUser) {
        // Set name in header and profile menu
        const nameElement = document.getElementById('userName') || document.getElementById('teacherName');
        if (nameElement) nameElement.textContent = currentUser.first_name;

        const profileName = document.getElementById('profileName');
        if (profileName) profileName.textContent = currentUser.first_name + ' ' + currentUser.last_name;

        // Set avatar initials
        const avatar = document.getElementById('profileAvatar');
        if (avatar) {
            const initials = (currentUser.first_name[0] || '') + (currentUser.last_name[0] || '');
            avatar.textContent = initials.toUpperCase();
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

    // Profile menu logic
    const profileMenu = document.getElementById('profileMenu');
    if (profileMenu) {
        profileMenu.addEventListener('click', function(e) {
            e.stopPropagation();
            profileMenu.classList.toggle('open');
        });
        // Close dropdown on outside click
        document.addEventListener('click', function() {
            profileMenu.classList.remove('open');
        });
    }
});

// Logout function
function logout() {
    localStorage.removeItem('authToken');
    localStorage.removeItem('currentUser');
    window.location.href = '/';
}

// Add this function for profile navigation
function openProfile() {
    // Optionally, navigate to a profile page or show a modal
    alert('Profile page coming soon!');
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
