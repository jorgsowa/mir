===file===
<?php
function outer(): void {
    isset($_GET['q']);

    $closure = function (): void {
        isset($_POST['name']);
    };

    $arrow = fn(): bool => isset($_SERVER['REQUEST_METHOD']);

    $closure();
    $arrow();
}
===expect===
