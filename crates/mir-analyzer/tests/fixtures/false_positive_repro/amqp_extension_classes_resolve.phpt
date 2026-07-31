===description===
FP-I1: the `amqp` PECL extension (AMQPConnection, AMQPChannel, ...) had no
vendored stubs/ dir despite PhpStormStubsMap.php already listing every
entry — same missing-stub root cause as the fixed C6 (ast).
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php

function connect(): AMQPConnection {
    return new AMQPConnection();
}

function openChannel(AMQPConnection $conn): AMQPChannel {
    return new AMQPChannel($conn);
}

function handle(AMQPException $e): string {
    return $e->getMessage();
}
===expect===
