===description===
A URL parsed with PHP 8.5's `Uri\Rfc3986\Uri` or `Uri\WhatWg\Url` keeps the
request taint of its input, so requests built from it are SSRF sinks.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <PossiblyNullMethodCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function whatwg(): void {
    $url = new Uri\WhatWg\Url($_GET['u']);
    curl_init($url->toAsciiString());
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}

function rfc3986(): void {
    $uri = Uri\Rfc3986\Uri::parse($_GET['u']);
    file_get_contents($uri->toString());
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
}

function schemeOnly(): void {
    $uri = new Uri\Rfc3986\Uri($_GET['u']);
    if ($uri->getScheme() === 'https') {
        curl_init($uri->toString());
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    }
}

function fixed(): void {
    $url = new Uri\WhatWg\Url('https://api.example.com/v1/');
    curl_init($url->toAsciiString());
}
