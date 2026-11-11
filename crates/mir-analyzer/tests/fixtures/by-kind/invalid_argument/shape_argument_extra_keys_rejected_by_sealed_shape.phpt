===description===
A shape parameter without `...` is sealed: an argument with a key it does not declare is `InvalidArgument`, from a literal or from a wider declared shape. A parameter spelled `array{host: string, ...}` still accepts extra keys, and an open argument may still be passed to a sealed one.
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
    //          ^^^^^^ InvalidArgument: Argument $config of takesConfig() expects 'array{'host': string, 'port': int}', got 'array{'host': string, 'port': int, 'timeout': int}'
    takesOpenConfig($wider);
}

/** @param array{host: string, port: int, ...} $open */
function passesOpen(array $open): void {
    takesConfig($open);
}
