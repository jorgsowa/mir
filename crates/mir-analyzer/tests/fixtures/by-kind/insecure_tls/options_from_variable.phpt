===description===
Option arrays and values built in a variable first are checked like literals.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <PossiblyInvalidArgument errorLevel="suppress"/>
    <PossiblyFalseArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fetch(\CurlHandle $ch, bool $verify): void {
    $options = ['ssl' => ['verify_peer' => false]];
    stream_context_create($options);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureTls: TLS verification disabled: ssl option verify_peer is off, so any certificate is accepted and anyone on the network can intercept the connection
    $curl = [CURLOPT_RETURNTRANSFER => true, CURLOPT_SSL_VERIFYPEER => false];
    curl_setopt_array($ch, $curl);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureTls: TLS verification disabled: CURLOPT_SSL_VERIFYPEER is off, so any certificate is accepted and anyone on the network can intercept the connection
    $off = 0;
    curl_setopt($ch, CURLOPT_SSL_VERIFYHOST, $off);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureTls: TLS verification disabled: CURLOPT_SSL_VERIFYHOST is off, so a certificate for any host is accepted and anyone on the network can intercept the connection
    curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, $verify);
}
