===description===
A URL whose scheme and host come from a literal local, a constant or a sprintf format can't be redirected to another host.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
const API = 'https://a.com/';

class Q {
    const BASE = 'https://a.com/';

    public function classConstant(): void {
        curl_init(self::BASE . $_GET['x']);
    }
}

function local(): void {
    $base = 'https://a.com/';
    curl_init($base . $_GET['x']);
}

function globalConstant(): void {
    curl_init(API . $_GET['x']);
}

function format(): void {
    curl_init(sprintf('https://a.com/%s', $_GET['x']));
}

function hostInFormat(): void {
    curl_init(sprintf('https://%s/', $_GET['x']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}

function noPathInLocal(): void {
    $base = 'https://a.com';
    curl_init($base . $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}

function unknownBase(string $base): void {
    curl_init($base . $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}
