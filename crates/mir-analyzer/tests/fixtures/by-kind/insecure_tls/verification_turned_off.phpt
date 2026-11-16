===description===
A literal false or 0 for curl's certificate or host checks, or for the `ssl`
stream context's `verify_peer`/`verify_peer_name`, lets anyone on the network
intercept the connection.
===config===
<mir>
  <issueHandlers>
    <PossiblyInvalidArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fetch(string $url): void {
    $ch = curl_init($url);
    curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, false);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureTls: TLS verification disabled: CURLOPT_SSL_VERIFYPEER is off, so any certificate is accepted and anyone on the network can intercept the connection
    curl_setopt($ch, \CURLOPT_SSL_VERIFYHOST, 0);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureTls: TLS verification disabled: CURLOPT_SSL_VERIFYHOST is off, so a certificate for any host is accepted and anyone on the network can intercept the connection
    curl_setopt_array($ch, [CURLOPT_RETURNTRANSFER => true, CURLOPT_PROXY_SSL_VERIFYPEER => false]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureTls: TLS verification disabled: CURLOPT_PROXY_SSL_VERIFYPEER is off, so any proxy certificate is accepted and anyone on the network can intercept the connection
    $context = stream_context_create(['ssl' => ['verify_peer' => false]]);
//             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureTls: TLS verification disabled: ssl option verify_peer is off, so any certificate is accepted and anyone on the network can intercept the connection
    stream_context_set_option($context, 'ssl', 'verify_peer_name', false);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureTls: TLS verification disabled: ssl option verify_peer_name is off, so a certificate for any host is accepted and anyone on the network can intercept the connection
    stream_context_set_option($context, ['ssl' => ['verify_peer' => 0]]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureTls: TLS verification disabled: ssl option verify_peer is off, so any certificate is accepted and anyone on the network can intercept the connection
}
