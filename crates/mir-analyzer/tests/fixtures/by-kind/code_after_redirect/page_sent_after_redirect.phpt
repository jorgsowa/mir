===description===
A redirect that isn't followed by `exit` still sends the rest of the page, so a
client that ignores the `Location` header reads the protected content.
===file===
<?php
function admin(bool $loggedIn): void {
    if (!$loggedIn) {
        header('Location: /login.php');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CodeAfterRedirect: Location redirect isn't followed by exit — the rest of the page is still sent
    }
    echo 'Secret admin page';
}

function moved(string $url): void {
    header("location: $url");
//  ^^^^^^^^^^^^^^^^^^^^^^^^ CodeAfterRedirect: Location redirect isn't followed by exit — the rest of the page is still sent
    print 'Moved';
}

function template(bool $allowed): void {
    if (!$allowed) {
        header('Location: ' . '/denied');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CodeAfterRedirect: Location redirect isn't followed by exit — the rest of the page is still sent
    }
    ?>
    <h1>Dashboard</h1>
    <?php
}
