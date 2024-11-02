document.addEventListener('DOMContentLoaded', function() {
    const menuButton = document.querySelector('.mobile-nav .hamburger');
    const mobileNav = document.querySelector('.sidebar');

    menuButton.addEventListener('click', function() {
        mobileNav.classList.toggle('open');
    });
});