===description===
curl_getinfo() returns an array when every info value is requested.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php

function fromHandle(CurlHandle $handle): void {
    $info = curl_getinfo($handle);
    /** @mir-check $info is array */
    $withError = ['errno' => 0] + $info;
}

function fromInit(): void {
    $handle = curl_init();
    if ($handle === false) {
        return;
    }

    $info = curl_getinfo($handle, null);
    /** @mir-check $info is array */
    $withTiming = ['total_time' => 0.0] + $info;
}

function selectedInfo(CurlHandle $handle): void {
    $info = curl_getinfo($handle, CURLINFO_RESPONSE_CODE);
    /** @mir-check $info is mixed */
}
===expect===
