===description===
open-resource rejects a handle closed by fclose(); plain resource and unknown handles are accepted
===file===
<?php
/** @param open-resource $value */
function acceptsOpenResource($value): void {
    echo get_resource_type($value);
}

/** @param resource $value */
function acceptsResource($value): void {
    echo gettype($value);
}

/** @param resource $value */
function forwardsUnknown($value): void {
    acceptsOpenResource($value);
}

/** @param open-resource $value */
function closesOpen($value): void {
    fclose($value);
    acceptsOpenResource($value);
//                      ^^^^^^ InvalidArgument: Argument $value of acceptsOpenResource() expects 'open-resource', got 'closed-resource'
}

function caller(): void {
    $handle = \fopen('php://memory', 'r');
    \assert($handle !== false);
    acceptsOpenResource($handle);
    acceptsOpenResource('not a resource');
//                      ^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of acceptsOpenResource() expects 'open-resource', got '"not a resource"'

    $closed = \fopen('php://memory', 'r');
    \assert($closed !== false);
    \fclose($closed);
    acceptsOpenResource($closed);
//                      ^^^^^^^ InvalidArgument: Argument $value of acceptsOpenResource() expects 'open-resource', got 'closed-resource'
    acceptsResource($closed);
}
