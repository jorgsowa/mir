===description===
`FILTER_VALIDATE_IP` flags held in a variable or options array built first
reject internal addresses like flags written at the call; without them the
address can still reach internal hosts.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function bothRanges(): void {
    $public = FILTER_FLAG_NO_PRIV_RANGE | FILTER_FLAG_NO_RES_RANGE;
    $ip = filter_var($_GET['h'], FILTER_VALIDATE_IP, $public);
    curl_init("http://$ip/");
}

function optionsArray(): void {
    $options = ['flags' => FILTER_FLAG_GLOBAL_RANGE];
    $ip = filter_var($_GET['h'], FILTER_VALIDATE_IP, $options);
    curl_init("http://$ip/");
}

function onlyIpv4(): void {
    $loose = FILTER_FLAG_IPV4;
    $ip = filter_var($_GET['h'], FILTER_VALIDATE_IP, $loose);
    curl_init("http://$ip/");
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}
