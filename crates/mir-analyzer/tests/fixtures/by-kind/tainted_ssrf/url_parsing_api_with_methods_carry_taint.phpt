===description===
`withHost()`, `resolve()` and the other `with*()` methods build a new URL from
their arguments, so request data passed to them reaches the request, even after
the original URL's host was checked.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <PossiblyNullMethodCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
use Uri\Rfc3986\Uri;
use Uri\WhatWg\Url;

function repointed(): void {
    $uri = Uri::parse('https://api.example.com/');
    $target = $uri->withHost($_GET['host']);
    curl_init($target->toString());
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}

function resolvedReference(): void {
    $url = new Url('https://api.example.com/');
    $target = $url->resolve($_GET['ref']);
    curl_init($target->toAsciiString());
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}

function repointedAfterCheck(): void {
    $uri = Uri::parse($_GET['url']);
    if ($uri?->getHost() === 'api.example.com') {
        $uri = $uri->withHost($_GET['host']);
        curl_init($uri->toString());
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    }
}

function fixedPath(): void {
    $uri = Uri::parse('https://api.example.com/');
    $target = $uri->withPath('/v1/status');
    curl_init($target->toString());
}

function chained(): void {
    $uri = Uri::parse('https://api.example.com/');
    curl_init($uri->withHost($_GET['host'])->toString());
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}

function chainedFromParse(): void {
    curl_init(Uri::parse('https://api.example.com/')?->withHost($_GET['host'])->toString());
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}
