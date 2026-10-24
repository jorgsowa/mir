===description===
A literal `scheme://host/` prefix fixes the destination, so appended request
data can't redirect an SSRF or file sink; it stays a sink when the host, scheme
or authority terminator is attacker-controlled or the scheme can reach local
files. `http_build_query()` output is encoded like `urlencode()`.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fixed_origin(CurlHandle $ch): void {
    $id = $_GET['id'];
    curl_init('https://api.example.com/users/' . urlencode($id));
    curl_init('https://api.example.com/users/' . $id);
    curl_init("https://api.example.com/users/{$id}");
    curl_init("https://api.example.com/?q=$id");
    curl_init(('https://api.example.com/' . $id) . '/x');
    curl_init('https://api.example.com/a' . '/' . $id . '?x=' . $id);
    $url = 'https://api.example.com/?' . http_build_query(['id' => $id]);
    curl_setopt($ch, CURLOPT_URL, $url);
    curl_setopt_array($ch, [CURLOPT_URL => 'https://api.example.com/?id=' . $id]);
    get_headers('https://x.example.com/?id=' . rawurlencode($id));
    file_get_contents('https://api.example.com/?q=' . urlencode($id));
    file_get_contents('ftp://files.example.com/' . $id);
    fopen('HTTPS://api.example.com:8443/x?' . $id, 'r');
    new SoapClient('https://api.example.com/' . $id . '.wsdl');
    curl_init(<<<URL
    https://api.example.com/$id
    URL);
}

function attacker_controls_destination(): void {
    $id = $_GET['id'];
    curl_init($id . 'https://api.example.com/x');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    curl_init('https://' . $id);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    curl_init('https://' . $id . '/x');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    curl_init('https://api.example.com' . $id);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    curl_init('https://api.example.com@' . $id);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    curl_init("https://$id/x");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    curl_init($id . '/x');
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    curl_init('/' . $id);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
    curl_init('https://' . urlencode($id));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
}

function local_schemes_stay_sinks(): void {
    $p = $_GET['p'];
    file_get_contents('file:///var/data/' . $p);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    file_get_contents('php://filter/resource=' . $p);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    file_get_contents('/var/data/' . $p);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    file_get_contents('https://api.example.com/?q=' . $p . '&z');
}

function other_kinds_survive(PDO $db): void {
    $q = $_GET['q'];
    echo 'https://api.example.com/?q=' . $q;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $db->query('https://api.example.com/?q=' . $q);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    echo 'https://api.example.com/?q=' . http_build_query(['q' => $q]);
}
