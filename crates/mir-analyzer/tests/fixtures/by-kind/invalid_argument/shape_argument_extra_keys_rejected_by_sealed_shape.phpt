===description===
A shape parameter without `...` rejects an array literal with a key it does not declare. A wider shape held in a variable is accepted, as is any argument for a parameter spelled `array{host: string, ...}`.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array{host: string, port: int} $config */
function takesConfig(array $config): void {}

/** @param array{host: string, ...} $config */
function takesOpenConfig(array $config): void {}

takesConfig(['host' => 'localhost', 'port' => 3306]);
takesConfig(['host' => 'localhost', 'port' => 3306, 'timeout' => 30]);
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $config of takesConfig() expects 'array{'host': string, 'port': int}', got 'array{'host': "localhost", 'port': 3306, 'timeout': 30}'
takesOpenConfig(['host' => 'localhost', 'port' => 3306, 'timeout' => 30]);

/** @param array{host: string, port: int, timeout: int} $wider */
function passesWider(array $wider): void {
    takesConfig($wider);
    takesOpenConfig($wider);
}

/** @param array{host: string, port: int, ...} $open */
function passesOpen(array $open): void {
    takesConfig($open);
}

$built = ['host' => 'localhost', 'port' => 3306, 'timeout' => 30];
takesConfig($built);
takesConfig(['host' => 'localhost']);
//          ^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $config of takesConfig() expects 'array{'host': string, 'port': int}', got 'array{'host': "localhost"}'
