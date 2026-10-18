===description===
The URL handed to curl through `CURLOPT_URL` or `CURLOPT_PROXY`, and a SOAP
client's WSDL location, are SSRF sinks. Other curl options are not.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fetch(CurlHandle $ch): void {
    curl_setopt($ch, CURLOPT_URL, $_GET['url']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    curl_setopt($ch, CURLOPT_PROXY, $_GET['proxy']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    curl_setopt_array($ch, [CURLOPT_URL => $_GET['url'], CURLOPT_TIMEOUT => 5]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    curl_setopt($ch, CURLOPT_USERAGENT, $_GET['agent']);
    curl_setopt($ch, CURLOPT_URL, 'https://api.example.com/');
    curl_setopt_array($ch, [CURLOPT_URL => 'https://api.example.com/', CURLOPT_POSTFIELDS => $_GET['body']]);
}

function soap(): void {
    new SoapClient($_GET['wsdl']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    new SoapClient('https://api.example.com/service.wsdl');
}
