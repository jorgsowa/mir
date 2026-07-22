===description===
A shape argument with EXTRA keys beyond the shape-typed parameter's
declared set is still accepted — the required-key check added to
`array_list_compatible`'s shape-to-shape arm only validates the param's
own declared keys, it doesn't reject unknown arg keys.
===config===
suppress=MissingParamType,UnusedParam
===file===
<?php
/** @param array{host: string, port: int} $config */
function takesConfig(array $config): void {}

takesConfig(['host' => 'localhost', 'port' => 3306, 'timeout' => 30]);
===expect===
