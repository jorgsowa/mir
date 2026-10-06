===description===
A shape argument missing a required key declared by the shape-typed
parameter is rejected — `array_list_compatible`'s shape-to-shape arm
previously returned unconditionally `true`, treating every shape as
compatible with every other shape regardless of required keys.
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

takesConfig(['host' => 'localhost', 'port' => 3306]);
takesConfig(['host' => 'localhost']);
//          ^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $config of takesConfig() expects 'array{'host': string, 'port': int}', got 'array{'host': "localhost"}'
