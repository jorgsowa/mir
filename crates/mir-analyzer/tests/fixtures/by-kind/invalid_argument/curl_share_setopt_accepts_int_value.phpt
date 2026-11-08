===description===
`curl_share_setopt()`'s `$value` is `mixed`, so a lock-data constant is accepted in strict mode.
===file===
<?php declare(strict_types=1);
function f(CurlShareHandle $sh): void { curl_share_setopt($sh, CURLSHOPT_SHARE, CURL_LOCK_DATA_DNS); }
