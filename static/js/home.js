document.addEventListener('DOMContentLoaded', function() {
  // Helper to get a cookie value by name
  function getCookie(name) {
    return document.cookie.split('; ').find(row => row.startsWith(name + '='))?.split('=')[1];
  }

  // If already authenticated, hide password input and auto-redirect to /master
  if (getCookie('auth') === '1') {
    document.getElementById('passwordInput').type = 'hidden'; // Hide password field
    document.getElementById('passwordInput').value = 'changeme'; // Optionally auto-fill
    document.getElementById('loginForm').onsubmit = function(e) {
      // Directly go to /master without submitting password
      window.location.href = '/master';
      e.preventDefault();
    };
  } else {
    // If not authenticated, handle login form submission
    document.getElementById('loginForm').onsubmit = async function(e) {
      e.preventDefault();
      document.getElementById('errorMsg').classList.add('hidden'); // Hide error message

      // Get password from input
      const password = document.getElementById('passwordInput').value;
      const params = new URLSearchParams();
      params.append('password', password);

      // Send login request to server
      const res = await fetch('/login', {
        method: 'POST',
        body: params,
        credentials: 'same-origin',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        }
      });

      // Handle server response
      if (res.redirected) {
        // Login successful, redirect to /master
        window.location.href = res.url;
      } else if (res.status === 401) {
        // Wrong password
        document.getElementById('errorMsg').textContent = 'Wrong access code';
        document.getElementById('errorMsg').classList.remove('hidden');
      } else {
        // Other error
        document.getElementById('errorMsg').textContent = 'Unexpected error';
        document.getElementById('errorMsg').classList.remove('hidden');
      }
    };
  }
});