===description===
An object passed as an ARGUMENT to a call may have its own properties
reassigned inside the callee, regardless of what the call's receiver is
(e.g. `$this->save($logger)` mutating `$logger`) — narrowing on an argument
must be dropped unless the callee is proven pure or (for methods)
`@psalm-external-mutation-free`. A free function has no equivalent
mutation-free-on-args signal, so only `@pure` is trusted for it.
===config===
suppress=UnusedVariable,MissingConstructor,UnusedParam
===file===
<?php
class Logger {
    public ?string $tag = null;
}

class Service {
    public function saveImpure(Logger $logger): void {
        if (rand(0, 1) === 1) {
            $logger->tag = null;
        }
    }

    /** @psalm-external-mutation-free */
    public function saveSafe(Logger $logger): void {
        // may read $logger, must not mutate it
    }
}

function mutateLogger(Logger $logger): void {
    if (rand(0, 1) === 1) {
        $logger->tag = null;
    }
}

/** @pure */
function readLogger(Logger $logger): ?string {
    return $logger->tag;
}

function methodArgInvalidates(Service $s, Logger $logger): void {
    $logger->tag = 'set';
    /** @mir-check $logger->tag is string */
    $_ = 1;
    $s->saveImpure($logger);
    /** @mir-check $logger->tag is string|null */
    $_ = 1;
}

function externalMutationFreeMethodArgPreserves(Service $s, Logger $logger): void {
    $logger->tag = 'set';
    $s->saveSafe($logger);
    /** @mir-check $logger->tag is string */
    $_ = 1;
}

function functionArgInvalidates(Logger $logger): void {
    $logger->tag = 'set';
    mutateLogger($logger);
    /** @mir-check $logger->tag is string|null */
    $_ = 1;
}

function pureFunctionArgPreserves(Logger $logger): void {
    $logger->tag = 'set';
    readLogger($logger);
    /** @mir-check $logger->tag is string */
    $_ = 1;
}
===expect===
