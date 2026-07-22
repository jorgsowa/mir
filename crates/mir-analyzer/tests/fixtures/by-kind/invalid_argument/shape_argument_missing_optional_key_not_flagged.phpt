===description===
A shape argument that omits an *optional* key declared by the shape-typed
parameter is still accepted — regression guard for the required-key check
added to `array_list_compatible`'s shape-to-shape arm, so it doesn't
over-tighten to require every declared key.
===config===
suppress=MissingParamType,UnusedParam
===file===
<?php
/** @param array{host: string, port?: int} $config */
function takesConfig(array $config): void {}

takesConfig(['host' => 'localhost']);
takesConfig(['host' => 'localhost', 'port' => 3306]);
===expect===
