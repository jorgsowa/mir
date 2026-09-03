===description===
@psalm-type alias defined on a standalone function does not produce false positives
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

class Status {
    public function __construct(public readonly string $value) {}
}

/**
 * @psalm-type StatusCode = "active"|"inactive"|"pending"
 * @param StatusCode $code
 * @return Status
 */
function makeStatus(string $code): Status {
    return new Status($code);
}

makeStatus("active");
makeStatus("inactive");
makeStatus("pending");
===expect===
