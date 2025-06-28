document.addEventListener('DOMContentLoaded', async function() {
  function getCookie(name) {
    return document.cookie.split('; ').find(row => row.startsWith(name + '='))?.split('=')[1];
  }

  const loginContainer = document.getElementById('loginContainer');

  // Check if password is required from backend
  const res = await fetch('/api/password_required');
  const passwordRequired = await res.json();

  if ((getCookie('auth') === '1') || (!passwordRequired)) {
    window.location.href = '/master';
  } else {
    // Show login form if password is required
    loginContainer.style.display = '';
    document.getElementById('loginForm').onsubmit = async function(e) {
      e.preventDefault();
      document.getElementById('errorMsg').classList.add('hidden');
      const password = document.getElementById('passwordInput').value;
      const params = new URLSearchParams();
      params.append('password', password);

      const res = await fetch('/login', {
        method: 'POST',
        body: params,
        credentials: 'same-origin',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        }
      });

      if (res.redirected) {
        window.location.href = res.url;
      } else if (res.status === 401) {
        document.getElementById('errorMsg').textContent = 'Wrong access code';
        document.getElementById('errorMsg').classList.remove('hidden');
      } else {
        document.getElementById('errorMsg').textContent = 'Unexpected error';
        document.getElementById('errorMsg').classList.remove('hidden');
      }
    };
  }
});
