===description===
The `grpc` and `opentelemetry` PECL extensions had no vendored stubs/ dir even though
PhpStormStubsMap.php lists their symbols, so every class, namespaced constant and
function was reported undefined.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

use Grpc\Channel;
use Grpc\ChannelCredentials;
use Grpc\Timeval;

function open(string $target): Channel {
    return new Channel($target, ['credentials' => ChannelCredentials::createInsecure()]);
}

function deadline(): Timeval {
    return Timeval::now();
}

function codes(): array {
    return [\Grpc\OP_SEND_MESSAGE, \Grpc\STATUS_OK, \Grpc\CALL_OK];
}

function instrument(): bool {
    return \OpenTelemetry\Instrumentation\hook(null, 'handle', static function (): void {});
}
