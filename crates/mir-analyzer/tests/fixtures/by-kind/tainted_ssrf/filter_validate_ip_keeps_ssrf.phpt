===description===
`FILTER_VALIDATE_IP` accepts loopback, private and metadata addresses, so the
result is still an SSRF target unless private and reserved ranges are rejected.
It stays safe for every other sink.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <PossiblyInvalidArrayAccess errorLevel="suppress"/>
    <PossiblyNullArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function plain(): void {
    $ip = filter_var($_GET['h'], FILTER_VALIDATE_IP);
    curl_init("http://$ip/");
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    echo $ip;
}

function onlyPrivate(): void {
    $ip = filter_var($_GET['h'], FILTER_VALIDATE_IP, FILTER_FLAG_NO_PRIV_RANGE);
    curl_init("http://$ip/");
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}

function input(): void {
    $ip = filter_input(INPUT_GET, 'h', FILTER_VALIDATE_IP);
    fsockopen($ip);
//  ^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}

function bothRanges(): void {
    $ip = filter_var($_GET['h'], FILTER_VALIDATE_IP, FILTER_FLAG_NO_PRIV_RANGE | FILTER_FLAG_NO_RES_RANGE);
    curl_init("http://$ip/");
}

function optionsArray(): void {
    $ip = filter_var($_GET['h'], FILTER_VALIDATE_IP, ['flags' => FILTER_FLAG_IPV4 | FILTER_FLAG_NO_PRIV_RANGE | FILTER_FLAG_NO_RES_RANGE]);
    curl_init("http://$ip/");
}

function globalRange(): void {
    $ip = filter_input(INPUT_GET, 'h', FILTER_VALIDATE_IP, FILTER_FLAG_GLOBAL_RANGE);
    curl_init("http://$ip/");
}

function definitionArray(): void {
    $in = filter_input_array(INPUT_GET, ['h' => ['filter' => FILTER_VALIDATE_IP, 'flags' => FILTER_FLAG_GLOBAL_RANGE]]);
    curl_init('http://' . $in['h'] . '/');
}
