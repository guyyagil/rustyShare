document.addEventListener('DOMContentLoaded', function() {
  // Check for auth cookie
  function getCookie(name) {
    return document.cookie.split('; ').find(row => row.startsWith(name + '='))?.split('=')[1];
  }
  if (getCookie('auth') === '1') {
    // Hide password input, show only Enter button
    document.getElementById('passwordInput').type = 'hidden';
    document.getElementById('passwordInput').value = 'changeme'; // auto-fill if you want
    document.getElementById('loginForm').onsubmit = function(e) {
      // Directly submit the form (simulate login)
      window.location.href = '/master';
      e.preventDefault();
    };
  } else {
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