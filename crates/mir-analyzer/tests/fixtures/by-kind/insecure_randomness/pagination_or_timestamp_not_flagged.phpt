===description===
Pagination tokens only locate data, timestamps returned under other names
aren't secrets, and clock values mixed with a secure generator are fine.
===file===
<?php
function nextPageToken(): string {
    return base64_encode((string) time());
}

function createdAt(): int {
    return time();
}

function apiToken(): string {
    return bin2hex(random_bytes(16)) . dechex(time());
}
