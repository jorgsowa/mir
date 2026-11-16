===description===
Verification left on, or set from a value mir can't see as a literal, isn't
reported; other options turned off aren't either.
===config===
<mir>
  <issueHandlers>
    <PossiblyInvalidArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fetch(string $url, bool $verify): void {
    $ch = curl_init($url);
    curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, true);
    curl_setopt($ch, CURLOPT_SSL_VERIFYHOST, 2);
    curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, $verify);
    curl_setopt($ch, CURLOPT_FOLLOWLOCATION, false);
    curl_setopt_array($ch, [CURLOPT_SSL_VERIFYPEER => $verify, CURLOPT_HEADER => 0]);
    $context = stream_context_create(['ssl' => ['verify_peer' => true, 'allow_self_signed' => false]]);
    stream_context_create(['http' => ['verify_peer' => false]]);
    stream_context_set_option($context, 'ssl', 'capture_peer_cert', false);
}
