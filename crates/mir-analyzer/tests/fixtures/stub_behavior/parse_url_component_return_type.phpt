===description===
`parse_url()` with a `PHP_URL_*` component returns that component, not the array.
===file===
<?php
function host(string $url): void {
    $host = parse_url($url, PHP_URL_HOST);
    /** @mir-check $host is string|null|false */
    echo (string) $host;
    $port = parse_url($url, PHP_URL_PORT);
    /** @mir-check $port is int|null|false */
    echo (int) $port;
}
