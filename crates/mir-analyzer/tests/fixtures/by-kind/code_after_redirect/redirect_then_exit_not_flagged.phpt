===description===
A redirect followed by `exit`, `return`, `throw` or a function that never returns,
a redirect with nothing after it, a redirect outside any condition followed by its
own page, redirects in every branch, and other headers aren't reported.
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

function movedPage(string $url): void {
    header("Location: $url");
    echo '<a href="' . htmlspecialchars($url) . '">Moved</a>';
}

function redirectInEveryBranch(bool $ok): void {
    if ($ok) {
        header('Location: /home');
    } else {
        header('Location: /login');
    }
    echo 'Redirecting';
}

function redirectOrPage(bool $ok): void {
    if (!$ok) {
        header('Location: /login');
    } else {
        echo 'Welcome';
    }
}
