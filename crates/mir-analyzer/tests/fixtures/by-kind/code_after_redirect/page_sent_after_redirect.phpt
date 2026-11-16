===description===
A guard that redirects without `exit` lets the code after it run for the rejected
request: a client that ignores the `Location` header reads the page, and its
actions still happen, even when an `exit` comes later.
===file===
<?php
function admin(bool $loggedIn): void {
    if (!$loggedIn) {
        header('Location: /login.php');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CodeAfterRedirect: Location redirect isn't followed by exit — the code after it still runs
    }
    echo 'Secret admin page';
}

function deletesAnyway(bool $isAdmin): void {
    if (!$isAdmin) {
        header("location: /login");
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ CodeAfterRedirect: Location redirect isn't followed by exit — the code after it still runs
    }
    unlink('/srv/cache.db');
    echo 'Deleted';
    exit;
}

function template(bool $allowed): void {
    if (!$allowed) {
        header('Location: ' . '/denied');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CodeAfterRedirect: Location redirect isn't followed by exit — the code after it still runs
    }
    ?>
    <h1>Dashboard</h1>
    <?php
}
