// Dashboard functionality
document.addEventListener('DOMContentLoaded', async function() {
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

    // Fetch and populate dashboard stats and classroom lists
    const token = localStorage.getItem('authToken');
    if (!currentUser || !token) return;

    // Teacher dashboard
    if (document.getElementById('teacherClassroomList')) {
        // Fetch teacher's classrooms
        const resp = await fetch('/api/dashboard', { headers: { 'Authorization': 'Bearer ' + token } });
        if (resp.ok) {
            const html = await resp.text();
            // Optionally parse and extract classroom data from HTML or use a dedicated API endpoint for JSON
            // For now, just clear the placeholder
            document.getElementById('teacherClassroomList').innerHTML = '';
        }
        // Set stats to 0 or fetch from API if available
        document.getElementById('statTotalStudents').textContent = '0';
        document.getElementById('statActiveClasses').textContent = '0';
        document.getElementById('statRating').textContent = '-';
    }

    // Student dashboard
    if (document.getElementById('studentClassroomList')) {
        // Fetch student's classrooms
        const resp = await fetch('/api/dashboard', { headers: { 'Authorization': 'Bearer ' + token } });
        if (resp.ok) {
            const html = await resp.text();
            // Optionally parse and extract classroom data from HTML or use a dedicated API endpoint for JSON
            // For now, just clear the placeholder
            document.getElementById('studentClassroomList').innerHTML = '';
        }
        // Set stats to 0 or fetch from API if available
        document.getElementById('statClassesThisWeek').textContent = '0';
        document.getElementById('statStudyTime').textContent = '0h';
        document.getElementById('statCurrentLevel').textContent = '-';
    }

    // --- Modal logic for Create Class ---
    window.openCreateClassModal = function() {
        document.getElementById('createClassModal').style.display = 'block';
    };
    window.closeCreateClassModal = function() {
        document.getElementById('createClassModal').style.display = 'none';
    };

    // --- Modal logic for Upload Material ---
    window.openUploadMaterialModal = function() {
        document.getElementById('uploadMaterialModal').style.display = 'block';
    };
    window.closeUploadMaterialModal = function() {
        document.getElementById('uploadMaterialModal').style.display = 'none';
    };

    // --- THEME SWITCHER ---
    function setThemeSwitchers(isDark) {
        // Main theme toggle (mobile topbar)
        const themeToggle = document.getElementById('themeToggle');
        if (themeToggle) themeToggle.checked = isDark;
        const themeLabel = document.getElementById('themeLabel');
        if (themeLabel) themeLabel.textContent = isDark ? '☀️' : '🌙';

        // Sidebar theme toggle (desktop)
        const themeToggleSidebar = document.getElementById('themeToggleSidebar');
        if (themeToggleSidebar) themeToggleSidebar.checked = isDark;
        const themeLabelSidebar = document.getElementById('themeLabelSidebar');
        if (themeLabelSidebar) themeLabelSidebar.textContent = isDark ? '☀️' : '🌙';
    }
    window.toggleTheme = function() {
        // Always sync both toggles
        const themeToggle = document.getElementById('themeToggle');
        const themeToggleSidebar = document.getElementById('themeToggleSidebar');
        let isDark = false;
        if (themeToggle && themeToggle.checked) isDark = true;
        if (themeToggleSidebar && themeToggleSidebar.checked) isDark = true;
        document.body.classList.toggle('dark-theme', isDark);
        setThemeSwitchers(isDark);
        localStorage.setItem('theme', isDark ? 'dark' : 'light');
    };
    // On load, set theme from localStorage
    if (localStorage.getItem('theme') === 'dark') {
        document.body.classList.add('dark-theme');
        setThemeSwitchers(true);
    } else {
        document.body.classList.remove('dark-theme');
        setThemeSwitchers(false);
    }
    // Sync toggles if either is changed
    const themeToggle = document.getElementById('themeToggle');
    const themeToggleSidebar = document.getElementById('themeToggleSidebar');
    if (themeToggle) {
        themeToggle.addEventListener('change', () => {
            if (themeToggleSidebar) themeToggleSidebar.checked = themeToggle.checked;
            window.toggleTheme();
        });
    }
    if (themeToggleSidebar) {
        themeToggleSidebar.addEventListener('change', () => {
            if (themeToggle) themeToggle.checked = themeToggleSidebar.checked;
            window.toggleTheme();
        });
    }
    // Show/hide sidebar theme switcher based on screen size
    function updateSidebarThemeSwitcher() {
        const sidebarThemeSwitcher = document.getElementById('sidebarThemeSwitcher');
        if (!sidebarThemeSwitcher) return;
        if (window.innerWidth > 900) {
            sidebarThemeSwitcher.style.display = 'flex';
        } else {
            sidebarThemeSwitcher.style.display = 'none';
        }
    }
    window.addEventListener('resize', updateSidebarThemeSwitcher);
    updateSidebarThemeSwitcher();

    // --- LESSONS LOGIC ---
    async function loadTeacherLessons() {
        const list = document.getElementById('teacherLessonList');
        if (!list) return;
        const token = localStorage.getItem('authToken');
        const resp = await fetch('/api/lesson', {
            headers: { 'Authorization': 'Bearer ' + token }
        });
        if (resp.ok) {
            const lessons = await resp.json();
            if (lessons.length === 0) {
                list.innerHTML = '<div style="color:#64748b;">No lessons scheduled. Click "Schedule Lesson" to add one.</div>';
            } else {
                list.innerHTML = lessons.map(lesson => `
                    <div class="lesson-card">
                        <h4>${lesson.title}</h4>
                        <p>${lesson.description}</p>
                        <p><b>Classroom:</b> ${lesson.classroom_id}</p>
                        <p><b>Scheduled:</b> ${new Date(lesson.scheduled_at).toLocaleString()}</p>
                        <div class="lesson-actions">
                            <button class="btn btn-outline" onclick="startClass('${lesson.classroom_id}')">Start</button>
                        </div>
                    </div>
                `).join('');
            }
        } else {
            list.innerHTML = '<div style="color:#ef4444;">Failed to load lessons.</div>';
        }
    }

    // --- SCHEDULE LOGIC ---
    async function loadTeacherSchedule() {
        const list = document.getElementById('teacherScheduleList');
        if (!list) return;
        const token = localStorage.getItem('authToken');
        const resp = await fetch('/api/lesson', {
            headers: { 'Authorization': 'Bearer ' + token }
        });
        if (resp.ok) {
            const lessons = await resp.json();
            const upcoming = lessons.filter(l => new Date(l.scheduled_at) > new Date());
            if (upcoming.length === 0) {
                list.innerHTML = '<div style="color:#64748b;">No upcoming lessons.</div>';
            } else {
                list.innerHTML = upcoming.map(lesson => `
                    <div class="schedule-card">
                        <h4>${lesson.title}</h4>
                        <p>${lesson.description}</p>
                        <p><b>When:</b> ${new Date(lesson.scheduled_at).toLocaleString()}</p>
                    </div>
                `).join('');
            }
        } else {
            list.innerHTML = '<div style="color:#ef4444;">Failed to load schedule.</div>';
        }
    }

    // --- CREATE LESSON MODAL LOGIC ---
    window.openCreateLessonModal = async function() {
        // Populate classroom dropdown
        const select = document.getElementById('lessonClassroom');
        select.innerHTML = '<option value="">Select Classroom</option>';
        const token = localStorage.getItem('authToken');
        const resp = await fetch('/api/classroom', {
            headers: { 'Authorization': 'Bearer ' + token }
        });
        if (resp.ok) {
            const classes = await resp.json();
            classes.forEach(cls => {
                const opt = document.createElement('option');
                opt.value = cls.id;
                opt.textContent = cls.name;
                select.appendChild(opt);
            });
        }
        document.getElementById('createLessonModal').style.display = 'block';
    };
    window.closeCreateLessonModal = function() {
        document.getElementById('createLessonModal').style.display = 'none';
    };

    // --- CREATE LESSON FORM SUBMISSION ---
    const createLessonForm = document.getElementById('createLessonForm');
    if (createLessonForm) {
        createLessonForm.onsubmit = async function(e) {
            e.preventDefault();
            const title = document.getElementById('lessonTitle').value.trim();
            const description = document.getElementById('lessonDescription').value.trim();
            const classroom_id = document.getElementById('lessonClassroom').value;
            const scheduled_at = document.getElementById('lessonDateTime').value;
            if (!title || !description || !classroom_id || !scheduled_at) return;
            const token = localStorage.getItem('authToken');
            const resp = await fetch('/api/lesson', {
                method: 'POST',
                headers: {
                    'Authorization': 'Bearer ' + token,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    id: crypto.randomUUID(),
                    classroom_id,
                    teacher_id: JSON.parse(localStorage.getItem('currentUser')).id,
                    title,
                    description,
                    scheduled_at,
                    is_active: true,
                    chat_closed: false,
                    created_at: new Date().toISOString()
                })
            });
            if (resp.ok) {
                closeCreateLessonModal();
                await loadTeacherLessons();
                await loadTeacherSchedule();
            } else {
                alert('Failed to schedule lesson.');
            }
        };
    }

    // --- MOBILE SIDEBAR SLIDE-IN LOGIC ---
    const sidebar = document.getElementById('sidebar');
    const sidebarToggleBtn = document.getElementById('sidebarToggleBtn');
    const sidebarBackdrop = document.getElementById('sidebarBackdrop');
    // Show toggle button on mobile
    function updateSidebarToggleVisibility() {
        if (window.innerWidth <= 900) {
            if (sidebarToggleBtn) sidebarToggleBtn.style.display = 'flex';
        } else {
            if (sidebarToggleBtn) sidebarToggleBtn.style.display = 'none';
            if (sidebar) sidebar.classList.remove('open');
            if (sidebarBackdrop) sidebarBackdrop.classList.remove('active');
        }
    }
    window.addEventListener('resize', updateSidebarToggleVisibility);
    updateSidebarToggleVisibility();

    if (sidebarToggleBtn && sidebar && sidebarBackdrop) {
        sidebarToggleBtn.addEventListener('click', function(e) {
            sidebar.classList.add('open');
            sidebarBackdrop.classList.add('active');
        });
        sidebarBackdrop.addEventListener('click', function() {
            sidebar.classList.remove('open');
            sidebarBackdrop.classList.remove('active');
        });
        // Close sidebar when a nav item is clicked (on mobile)
        sidebar.querySelectorAll('.nav-item').forEach(item => {
            item.addEventListener('click', function() {
                if (window.innerWidth <= 900) {
                    sidebar.classList.remove('open');
                    sidebarBackdrop.classList.remove('active');
                }
            });
        });
    }

    // Initial load for teacher dashboard
    if (document.getElementById('teacherClassroomList')) {
        await loadTeacherClassrooms();
    }
    if (document.getElementById('teacherMaterialList')) {
        await loadTeacherMaterials();
    }
    if (document.getElementById('teacherLessonList')) {
        await loadTeacherLessons();
    }
    if (document.getElementById('teacherScheduleList')) {
        await loadTeacherSchedule();
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
    // Try to get the current classroom's Zoom join URL and open it
    const classroomId = getCurrentClassroomId(); // Implement this to get selected classroom
    const token = localStorage.getItem('authToken');
    fetch(`/api/classroom/${classroomId}/zoom/join`, {
        headers: { 'Authorization': 'Bearer ' + token }
    })
    .then(res => res.json())
    .then(data => {
        if (data.join_url) {
            window.open(data.join_url, '_blank');
        } else {
            alert('No live Zoom meeting available for this class.');
        }
    });
}

// Student requests a Zoom meeting
function requestZoomMeeting(classroomId) {
    const token = localStorage.getItem('authToken');
    fetch(`/api/classroom/${classroomId}/meeting-requests`, {
        method: 'POST',
        headers: { 'Authorization': 'Bearer ' + token }
    })
    .then(res => {
        if (res.status === 201) {
            alert('Meeting request sent to teacher.');
        } else {
            alert('Failed to request meeting.');
        }
    });
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
