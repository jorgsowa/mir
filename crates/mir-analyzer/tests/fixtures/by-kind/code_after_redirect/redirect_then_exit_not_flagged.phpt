===description===
A redirect followed by `exit`, `return`, `throw` or a function that never returns,
a redirect with no output after it, and other headers aren't reported.
===file===
<?php
/** @return never */
function stop(): never {
    exit;
}

function admin(bool $loggedIn): void {
    if (!$loggedIn) {
        header('Location: /login.php');
        exit;
    }
    echo 'Secret admin page';
}

function early(bool $ok): void {
    if (!$ok) {
        header('Location: /');
        http_response_code(302);
        return;
    }
    echo 'Welcome';
}

function viaHelper(bool $ok): void {
    if (!$ok) {
        header('Location: /');
        stop();
    }
    echo 'Welcome';
}

function thrown(bool $ok): void {
    if (!$ok) {
        header('Location: /');
        throw new RuntimeException('redirected');
    }
    echo 'Welcome';
}

function redirectOnly(string $url): void {
    header('Location: ' . $url);
}

function otherHeader(): void {
    header('Content-Type: text/plain');
    echo 'text';
}

function closureLater(): Closure {
    header('Location: /next');
    return function (): void {
        echo 'later';
    };
}
