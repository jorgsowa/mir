===description===
With named arguments, TooFewArguments counts the required parameters that got a value,
not every argument passed.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function connect(string $host, int $port, ?string $user = null, ?string $password = null): void {}

connect(port: 3306, user: 'root', password: 'secret');
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for connect(): expected 2, got 1
connect(user: 'root');
//<^^^^^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for connect(): expected 2, got 0
connect('db', user: 'root');
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for connect(): expected 2, got 1
connect(port: 3306, host: 'db');
connect('db', 3306, password: 'secret');

class Pool {
    public function __construct(public string $dsn, public int $size = 1) {}
    public function take(int $timeout, bool $block = true): void {}
}

$pool = new Pool(size: 4);
//      ^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for Pool::__construct(): expected 1, got 0
$pool->take(block: false);
//<^^^^^^^^^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for take(): expected 1, got 0
