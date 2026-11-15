===description===
Comparing the host a URL parser returned with literal hosts fixes where the
request goes, so it clears SSRF taint. The parsers resolve the host the same
way the request does, unlike a string prefix check.
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
function exact(): void {
    $uri = Uri\Rfc3986\Uri::parse($_GET['u']);
    if ($uri !== null && $uri->getHost() === 'api.example.com') {
        curl_init($uri->toString());
    }
}

function allowList(): void {
    $url = new Uri\WhatWg\Url($_GET['u']);
    if (!in_array($url->getAsciiHost(), ['api.example.com', 'cdn.example.com'], true)) {
        return;
    }
    curl_init($url->toAsciiString());
}

function otherHost(): void {
    $url = new Uri\WhatWg\Url($_GET['u']);
    if ($url->getAsciiHost() !== 'api.example.com') {
        curl_init($url->toAsciiString());
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    }
}
