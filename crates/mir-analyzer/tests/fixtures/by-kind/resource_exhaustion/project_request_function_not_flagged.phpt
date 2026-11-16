===description===
A project's own function named `request` isn't Laravel's helper.
===file===
<?php
function request(string $key): int {
    return strlen($key);
}

function pad(): void {
    echo str_repeat('-', request('width'));
}
