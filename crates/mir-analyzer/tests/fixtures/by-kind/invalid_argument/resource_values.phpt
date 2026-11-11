===description===
A docblock `resource` (also `open-resource` / `closed-resource`) is its own type: scalars and
objects aren't resources, and a resource isn't a string, int or bool. `is_resource()` narrows
`mixed` to it but its false branch keeps a resource, which may be closed. Stub handles whose native
type is a class (`CurlHandle`) don't inherit the docblock's `resource`, and a legacy resource
handle may still be passed to them.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @param resource $r */
function takes_resource($r): void {}
/** @param open-resource $r */
function takes_open($r): void {}
function takes_string(string $s): void {}

/** @param CurlHandle|resource|null $legacy */
function run(mixed $m, CurlHandle $c, ?object $o, $legacy): void {
    $h = fopen('php://memory', 'r');
    if ($h === false) {
        return;
    }
    /** @mir-check $h is resource */
    takes_resource($h);
    takes_open($h);
    echo (int) $h, (string) $h, gettype($h);
    takes_string($h);
//               ^^ InvalidArgument: Argument $s of takes_string() expects 'string', got 'resource'
    takes_resource('php://memory');
//                 ^^^^^^^^^^^^^^ InvalidArgument: Argument $r of takes_resource() expects 'resource', got '"php://memory"'
    takes_resource(new stdClass());
//                 ^^^^^^^^^^^^^^ InvalidArgument: Argument $r of takes_resource() expects 'resource', got 'stdClass'
    fclose($h);
    if (!is_resource($h)) {
//      ^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the check is redundant
        echo 'closed';
    }
    if (is_resource($m)) {
        /** @mir-check $m is resource */
        fclose($m);
    }
    $curl = curl_init();
    if ($curl !== false) {
        /** @mir-check $curl is CurlHandle */
        curl_close($curl);
    }
    curl_close($c);
    curl_close($o);
//             ^^ PossiblyNullArgument: Argument $handle of curl_close() might be null
    curl_close($legacy);
//             ^^^^^^^ PossiblyNullArgument: Argument $handle of curl_close() might be null
}
