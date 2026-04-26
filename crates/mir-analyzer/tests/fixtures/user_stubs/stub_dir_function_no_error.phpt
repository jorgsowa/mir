===config===
stub_dir=stubs
===file:stubs/auth.php===
<?php
function auth_check(string $token): bool { return strlen($token) > 0; }
===file:stubs/cache.php===
<?php
function cache_get(string $key): mixed { return null; }
===file:App.php===
<?php
function handle(string $token, string $key): void {
    if (auth_check($token)) {
        $val = cache_get($key);
    }
}
===expect===
