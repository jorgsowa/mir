===description===
extract() defines variables at runtime from the passed array, so later reads must
not be reported as UndefinedVariable; unrelated scopes are unaffected.
===file===
<?php
function getDsn(array $config): string {
    extract($config, EXTR_SKIP);
    $port = isset($config['port']) ? ',' . $port : '';
    return "host={$host}{$port};db={$database}";
}

function noDynamic(): void {
    echo $undefined;
//       ^^^^^^^^^^ UndefinedVariable: Variable $undefined is not defined
}
