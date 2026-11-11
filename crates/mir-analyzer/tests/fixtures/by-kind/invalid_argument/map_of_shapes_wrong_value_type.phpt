===description===
A shape nested in a map or in another shape is checked against the expected shape.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, array{host: string, port: int}> $servers */
function connect(array $servers): void {}
/** @param array{primary: array{host: string, port: int}} $config */
function configure(array $config): void {}

connect(['db' => ['host' => 'localhost', 'port' => '3306']]);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $servers of connect() expects 'array<string, array{'host': string, 'port': int}>', got 'array{'db': array{'host': "localhost", 'port': "3306"}}'
configure(['primary' => ['host' => 'localhost', 'port' => '3306']]);
//        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $config of configure() expects 'array{'primary': array{'host': string, 'port': int}}', got 'array{'primary': array{'host': "localhost", 'port': "3306"}}'
