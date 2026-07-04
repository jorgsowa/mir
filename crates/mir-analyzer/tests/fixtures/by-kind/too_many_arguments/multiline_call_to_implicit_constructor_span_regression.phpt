===description===
Regression: TooManyArguments against a class with no declared __construct
(expected 0) used to compute a nonsensical end column when the `new` call
spans multiple lines — the cross-line `.max(col_start + 1)` clamp compared
columns from different lines, producing a col_end past the end of the actual
closing-paren line. The span must end at the real closing paren, not at
col_start + 1 on the last line.
===file===
<?php
class Logger {}

class UnpackerS3Client
{
}

class Config
{
    public string $amazonKey = 'key';
    public string $amazonSecret = 'secret';
    public string $amazonRegion = 'region';
    public string $amazonMigrationBucket = 'bucket';
}

function make(Config $configuration, Logger $logger): UnpackerS3Client
{
    return new UnpackerS3Client(
        $logger,
        $configuration->amazonKey,
        $configuration->amazonSecret,
        $configuration->amazonRegion,
        $configuration->amazonMigrationBucket,
        binary: '/usr/bin/aws',
    );
}
===expect===
TooManyArguments@18:11-25:5: Too many arguments for UnpackerS3Client::__construct(): expected 0, got 6
