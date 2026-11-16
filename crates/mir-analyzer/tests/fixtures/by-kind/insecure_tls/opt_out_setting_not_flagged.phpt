===description===
Verification turned off only under a condition on a verify or SSL setting
honours the caller's opt-out and isn't reported; after the condition, an
unconditional switch-off is reported again.
===config===
<mir>
  <issueHandlers>
    <PossiblyInvalidArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, mixed> $options */
function transport(\CurlHandle $ch, array $options, bool $sslVerify): void {
    if ($options['verify'] === false) {
        curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, 0);
    }
    if (!$sslVerify) {
        curl_setopt($ch, CURLOPT_SSL_VERIFYHOST, 0);
    }
    curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, false);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureTls: TLS verification disabled: CURLOPT_SSL_VERIFYPEER is off, so any certificate is accepted and anyone on the network can intercept the connection
}
