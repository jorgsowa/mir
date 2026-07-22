===description===
A shape argument that HAS the required key but with an incompatible value
type is still rejected — the shape-to-shape required-key check must
validate value compatibility, not just key presence.
===config===
suppress=MissingParamType,UnusedParam
===file===
<?php
/** @param array{host: string, port: int} $config */
function takesConfig(array $config): void {}

takesConfig(['host' => 'localhost', 'port' => 'not-a-port']);
===expect===
InvalidArgument@5:12-5:59: Argument $config of takesConfig() expects 'array{'host': string, 'port': int}', got 'array{'host': "localhost", 'port': "not-a-port"}'
